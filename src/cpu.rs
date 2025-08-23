use core::time;
use std::collections::HashSet;

use enum_primitive_derive::Primitive;
use goblin::elf::Elf;
use tracing::debug;
use tracing::info;
use tracing::warn;

use self::alu::exec;
use self::decoder::decode;
use self::regfile::RegFile;

use crate::bus::{Bus, BusDevice, BusError};
use crate::cpu::csr::{ArchCSRs, CSRFile};
use crate::cpu::instructions::pretty_register;
use crate::trap::RVException;
use crate::RAM_SIZE;

pub mod alu;
pub mod csr;
pub mod decoder;
pub mod instructions;
pub mod regfile;

struct MMIORegister {
    value: u32,
    writable: bool,
}

#[derive(Debug, Clone, PartialEq, Primitive)]
enum ExecMode {
    MACHINE = 0b11,
    USER = 0b00,
}

pub struct Cpu {
    regfile: RegFile,
    csrfile: CSRFile,
    bus: Bus,
    amoreserved: HashSet<usize>,
    mode: ExecMode,
    pub pc: usize,
    pub delay: u64,
    pub count: u64,
}

const RAM_START: usize = 0x8000_0000;

impl Cpu {
    pub fn new(mut kernel: Vec<u8>, ram_size: usize) -> Self {
        if kernel.len() > ram_size {
            panic!("Kernel size exceeds RAM size");
        }
        info!(
            "Loading binary at {:#10x} with size {}",
            RAM_START,
            kernel.len()
        );
        kernel.extend(vec![0u8; ram_size - kernel.len()]);

        Self {
            regfile: RegFile::new(),
            csrfile: CSRFile::new(),
            bus: Bus::new(kernel, RAM_START),
            amoreserved: HashSet::new(),
            mode: ExecMode::MACHINE,
            pc: RAM_START,
            delay: 0,
            count: 0,
        }
    }

    pub fn load_dtb(&mut self, dtb_bytes: Vec<u8>) {
        let dtb_start = self.bus.ram.addr_space().1 - dtb_bytes.len();
        info!(
            "Loading DTB at {:#10x} with size {}",
            dtb_start,
            dtb_bytes.len()
        );
        self.bus.ram.mem[RAM_SIZE - dtb_bytes.len()..].copy_from_slice(&dtb_bytes);
        self.regfile.write(10, 0); // hartid
        self.regfile.write(11, dtb_start as i32); // DTB pointer
    }

    pub fn load_elf(&mut self, elf_bytes: Vec<u8>) {
        let elf = Elf::parse(&elf_bytes).unwrap();

        // Find all sections starting with .text
        for section in elf.section_headers.iter() {
            if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
                if name.starts_with(".text") || name.starts_with(".data") {
                    let offset = section.sh_offset as usize;
                    let size = section.sh_size as usize;
                    let addr = section.sh_addr as usize;
                    let text_bytes = &elf_bytes[offset..offset + size];
                    info!(
                        "Loading {} section at {:#08x} with size {}",
                        name, addr, size
                    );
                    self.bus.ram.mem[addr - RAM_START..addr - RAM_START + size]
                        .copy_from_slice(text_bytes);
                }
            }
        }
    }

    pub fn fetch(&self) -> Result<u32, RVException> {
        self.bus.load::<u32>(self.pc).map_err(|e| match e {
            BusError::AddressMisaligned(addr) => RVException::InstructionAddressMisaligned(addr),
            BusError::AddressUnmapped(addr) => RVException::InstructionAccessFault(addr),
        })
    }

    fn trap_entry(&mut self, exception: RVException) {
        info!("{:#010x} | Exception {:?}", self.pc, exception);

        // Disable interrupts
        self.csrfile.disable_irq();

        let mtval = match exception {
            RVException::InstructionAddressMisaligned(addr) => addr as u32,
            RVException::InstructionAccessFault(addr) => addr as u32,
            RVException::IllegalInstruction(instruction) => instruction,
            RVException::LoadAddressMisaligned(addr) => addr as u32,
            RVException::LoadAccessFault(addr) => addr as u32,
            RVException::StoreAddressMisaligned(addr) => addr as u32,
            RVException::StoreAccessFault(addr) => addr as u32,
            _ => 0,
        };

        // ECALL from riscv-tests test environment
        if exception == RVException::EnvironmentCall && self.regfile.read(17) == 93 {
            let result = self.regfile.read(10);
            info!("Test Result in a0: {}", result);
            std::process::exit(result);
        }

        self.csrfile.write(ArchCSRs::mtval as i32, mtval as i32);
        self.csrfile
            .write(ArchCSRs::mcause as i32, exception.to_ecode() as i32);

        self.csrfile.write(ArchCSRs::mepc as i32, self.pc as i32);
        self.pc = self.csrfile.read(ArchCSRs::mtvec as i32) as u32 as usize
    }

    fn trap_exit(&mut self) {
        // Re-enable interrupts
        self.csrfile.enable_irq();

        // Restore PC from mepc
        self.pc = self.csrfile.read(ArchCSRs::mepc as i32) as u32 as usize - 4;
    }

    fn next_instruction(&mut self) -> Result<(), RVException> {
        // Update CLINT
        self.bus.clint.tick(&mut self.csrfile);
        // Raise timer interrupt if enabled
        self.csrfile.mtimer_interrupt()?;

        // Fetch
        let instruction = self.fetch()?;
        // Decode
        let decoded_instr = decode(&instruction)?;
        info!(
            "{:#010x}| {:#010x} | {}",
            self.pc, instruction, decoded_instr
        );

        // Execute
        exec(self, decoded_instr)?;
        Ok(())
    }

    pub fn step(&mut self) {
        match self.next_instruction() {
            Err(exception) => self.trap_entry(exception),
            Ok(()) => self.pc += 4,
        };
        std::thread::sleep(time::Duration::from_millis(self.delay));
    }

    #[allow(dead_code)]
    pub fn dump_state(&self) {
        println!("=== CPU State @ PC {:#08x} ===", self.pc);
        for i in 0..32 {
            if i % 5 == 0 && i != 0 {
                println!("");
            }
            print!(
                "{:<4}: {:#010x}  ",
                pretty_register(&i),
                self.regfile.read(i)
            );
        }
        println!("");
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_srai() {
        let ram: Vec<u8> = [0xfffff0b7, 0x4010d093]
            .iter()
            .flat_map(|&v: &u32| v.to_le_bytes())
            .collect();
        let mut cpu = Cpu::new(ram, 1024);
        assert_eq!(cpu.next_instruction(), Ok(()));
        cpu.pc += 4;
        assert_eq!(cpu.next_instruction(), Ok(()));
        assert_eq!(cpu.regfile.read(1) as i32, -2048);
    }
    #[test]
    fn test_lui() {
        let ram: Vec<u8> = [0x800000b7, 0xfffff137]
            .iter()
            .flat_map(|&v: &u32| v.to_le_bytes())
            .collect();
        let mut cpu = Cpu::new(ram, 1024);
        assert_eq!(cpu.next_instruction(), Ok(()));
        assert_eq!(cpu.regfile.read(1) as u32, 0x80000000);
        cpu.pc += 4;
        assert_eq!(cpu.next_instruction(), Ok(()));
        assert_eq!(cpu.regfile.read(2) as u32, 0xfffff000);
    }
}
