use crate::cpu::decoder::DecodingError;

#[derive(Debug, Clone, PartialEq)]
pub enum RVException {
    InstructionAddressMisaligned(usize),
    InstructionAccessFault(usize),
    IllegalInstruction(DecodingError),
    BreakPoint,
    LoadAddressMisaligned(usize),
    LoadAccessFault(usize),
    StoreAddressMisaligned(usize),
    StoreAccessFault(usize),
    EnvironmentCall,
}

impl RVException {
    pub fn to_ecode(self) -> i32 {
        match self {
            Self::InstructionAddressMisaligned(_) => 0,
            Self::InstructionAccessFault(_) => 1,
            Self::IllegalInstruction(_) => 2,
            Self::BreakPoint => 3,
            Self::LoadAddressMisaligned(_) => 4,
            Self::LoadAccessFault(_) => 5,
            Self::StoreAddressMisaligned(_) => 6,
            Self::StoreAccessFault(_) => 7,
            Self::EnvironmentCall => 11,
        }
    }
}
