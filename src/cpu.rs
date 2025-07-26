use core::time;

use self::alu::exec;
use self::decoder::decode;
use self::regfile::RegFile;

use crate::bus::{Bus, BusDevice, BusError};
use crate::cpu::csr::{ArchCSRs, CSRFile};
use crate::exceptions::RVException;

pub mod alu;
pub mod csr;
pub mod decoder;
pub mod instructions;
pub mod regfile;

pub struct Cpu {
    regfile: RegFile,
    bus: Bus,
    csrfile: CSRFile,
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
        self.csrfile.write(ArchCSRs::Mepc as i32, self.pc as i32);
        self.csrfile
            .write(ArchCSRs::Mcause as i32, exception.to_ecode());
        self.pc = self.csrfile.read(ArchCSRs::Mtvec as i32) as usize
    }

    fn next_instruction(&mut self) -> Result<(), RVException> {
        // Fetch
        let instruction = self.fetch()?;
        println!("{:#08x}", instruction);
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
