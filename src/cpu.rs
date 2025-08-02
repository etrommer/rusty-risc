use core::time;
use std::collections::HashSet;

use self::alu::exec;
use self::decoder::decode;
use self::regfile::RegFile;

use crate::bus::{Bus, BusDevice, BusError};
use crate::cpu::csr::{ArchCSRs, CSRFile};
use crate::cpu::instructions::pretty_register;
use crate::trap::RVException;

pub mod alu;
pub mod csr;
pub mod decoder;
pub mod instructions;
pub mod regfile;

struct MMIORegister {
    value: u32,
    writable: bool,
}

pub struct Cpu {
    regfile: RegFile,
    csrfile: CSRFile,
    bus: Bus,
    amoreserved: HashSet<usize>,
    pub pc: usize,
    pub delay: u64,
}

impl Cpu {
    pub fn new(ram: Vec<u8>) -> Self {
        Self {
            regfile: RegFile::new(),
            csrfile: CSRFile::new(),
            bus: Bus::new(ram),
            amoreserved: HashSet::new(),
            pc: 0x80000000,
            delay: 0,
        }
    }

    pub fn fetch(&self) -> Result<u32, RVException> {
        self.bus.load::<u32>(self.pc).map_err(|e| match e {
            BusError::AddressMisaligned(addr) => RVException::InstructionAddressMisaligned(addr),
            BusError::AddressUnmapped(addr) => RVException::InstructionAccessFault(addr),
        })
    }

    fn trap_entry(&mut self, exception: RVException) {
        println!("Exception {:?} @ {:#08x}", exception, self.pc);

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
            println!("Test Result in a0: {}", result);
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

        let instruction = self.fetch()?;
        // println!("{:#08x}", instruction);
        // Decode
        let decoded_instr = decode(&instruction)?;
        println!("{:#08x} | {}", self.pc, decoded_instr);
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

    pub fn dump_state(&self) {
        println!("PC: {:#08x}", self.pc);
        println!("Registers:");
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
    fn test_lui() {
        let ram: Vec<u8> = [0x800000b7, 0xfffff137]
            .iter()
            .flat_map(|&v: &u32| v.to_le_bytes())
            .collect();
        let mut cpu = Cpu::new(ram);
        assert_eq!(cpu.next_instruction(), Ok(()));
        assert_eq!(cpu.regfile.read(1) as u32, 0x80000000);
        cpu.pc += 4;
        assert_eq!(cpu.next_instruction(), Ok(()));
        assert_eq!(cpu.regfile.read(2) as u32, 0xfffff000);
    }
}
