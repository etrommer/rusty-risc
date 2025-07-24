use self::alu::exec;
use self::decoder::decode;
use self::regfile::RegFile;

use crate::bus::{Bus, BusDevice, BusError};
use crate::exceptions::{self, RVException};

pub mod alu;
pub mod decoder;
pub mod instructions;
pub mod regfile;

pub struct Cpu {
    regfile: RegFile,
    bus: Bus,
    pc: usize,
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            regfile: RegFile::new(),
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
        match exception {
            RVException::InstructionAddressMisaligned(addr) => todo!(),
            RVException::InstructionAccessFault(addr) => todo!(),
            RVException::IllegalInstruction(error) => todo!(),
            RVException::BreakPoint => todo!(),
            RVException::LoadAddressMisaligned(addr) => todo!(),
            RVException::LoadAccessFault(addr) => todo!(),
            RVException::StoreAddressMisaligned(addr) => todo!(),
            RVException::StoreAccessFault(addr) => todo!(),
            RVException::EnvironmentCall => todo!(),
        }
    }
    fn next_instruction(&mut self) -> Result<(), RVException> {
        // Fetch
        let next_instruction = self.fetch()?;
        // Decode
        let decoded_instr = decode(&next_instruction)?;
        // Execute
        exec(self, decoded_instr)?;
        Ok(())
    }

    pub fn step(&mut self) {
        match self.next_instruction() {
            Err(exception) => self.handle_exception(exception),
            Ok(()) => self.pc += 4,
        };
    }
}
