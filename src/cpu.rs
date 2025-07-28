use core::time;

use self::alu::exec;
use self::decoder::decode;
use self::regfile::RegFile;

use crate::bus::{Bus, BusDevice, BusError};
use crate::cpu::csr::{ArchCSRs, CSRFile};
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
    pub pc: usize,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            regfile: RegFile::new(),
            csrfile: CSRFile::new(),
            bus,
            pc: 0x80000000,
        }
    }

    pub fn fetch(&self) -> Result<u32, RVException> {
        self.bus.load::<u32>(self.pc).map_err(|e| match e {
            BusError::AddressMisaligned(addr) => RVException::InstructionAddressMisaligned(addr),
            BusError::AddressUnmapped(addr) => RVException::InstructionAccessFault(addr),
        })
    }

    fn handle_exception(&mut self, exception: RVException) {
        // match exception {
        //     RVException::InstructionAddressMisaligned(addr) => todo!(),
        //     RVException::InstructionAccessFault(addr) => todo!(),
        //     RVException::IllegalInstruction(error) => todo!(),
        //     RVException::BreakPoint => todo!(),
        //     RVException::LoadAddressMisaligned(addr) => todo!(),
        //     RVException::LoadAccessFault(addr) => todo!(),
        //     RVException::StoreAddressMisaligned(addr) => todo!(),
        //     RVException::StoreAccessFault(addr) => todo!(),
        //     RVException::EnvironmentCall => todo!(),
        // };
        println!("Exception {:?} @ {:#08x}", exception, self.pc);
        self.csrfile.write(ArchCSRs::mepc as i32, self.pc as i32);
        self.csrfile
            .write(ArchCSRs::mcause as i32, exception.to_ecode() as i32);
        self.pc = self.csrfile.read(ArchCSRs::mtvec as i32) as usize
    }

    fn next_instruction(&mut self) -> Result<(), RVException> {
        // Update system timer
        // TODO: Should not always trap
        self.bus.clint.tick()?;

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
            Err(exception) => self.handle_exception(exception),
            Ok(()) => self.pc += 4,
        };
        std::thread::sleep(time::Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_code_and_get_cpu(code: &[u32]) -> Cpu {
        let code_bytes = code
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();
        let bus = Bus::new(code_bytes);
        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x80000000;
        for _ in 0..code.len() {
            cpu.step();
        }
        cpu
    }

    #[test]
    fn test_addi() {
        let code = [
            0x00100693_u32, // li a3,1
            0x00168713_u32, // addi a4,a3,1
        ];
        let cpu = run_code_and_get_cpu(&code);
        assert_eq!(cpu.regfile.read(14), 2);
    }

    #[test]
    fn test_add() {
        let code = [
            0x00100693_u32, // li a3,1
            0x00200713_u32, // li a4,2
            0x00e707b3_u32, // add a5,a4,a3 (a5 = a4 + a3)
        ];
        let cpu = run_code_and_get_cpu(&code);
        assert_eq!(cpu.regfile.read(15), 3); // a5 should be 3
    }

    #[test]
    fn test_andi() {
        let code = [
            0x00f00693_u32, // li a3,15
            0x00a6f713_u32, // andi a4,a3,10 (a4 = a3 & 10)
        ];
        let cpu = run_code_and_get_cpu(&code);
        assert_eq!(cpu.regfile.read(14), 10); // a4 should be 10
    }

    #[test]
    fn test_and() {
        let code = [
            0x00f00693_u32, // li a3,15
            0x00a00713_u32, // li a4,10
            0x00e707b3_u32, // and a5,a4,a3 (a5 = a4 & a3)
        ];
        let cpu = run_code_and_get_cpu(&code);
        assert_eq!(cpu.regfile.read(15), 10); // a5 should be 10
    }

    #[test]
    fn test_auipc() {
        let code = [
            0x00000297_u32, // auipc t0,0
        ];
        let cpu = run_code_and_get_cpu(&code);
        // auipc t0,0: t0 = pc + 0
        assert_eq!(cpu.regfile.read(5) as u32, 0x80000000_u32); // t0 should be initial pc
    }
}
