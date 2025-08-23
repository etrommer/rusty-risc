#[derive(Debug, Clone, PartialEq)]
pub enum RVException {
    InstructionAddressMisaligned(usize),
    InstructionAccessFault(usize),
    IllegalInstruction(u32),
    BreakPoint,
    LoadAddressMisaligned(usize),
    LoadAccessFault(usize),
    StoreAddressMisaligned(usize),
    StoreAccessFault(usize),
    EnvironmentCallU,
    EnvironmentCallM,
    TimerInterrupt,
}

impl RVException {
    pub fn to_ecode(&self) -> u32 {
        match self {
            Self::InstructionAddressMisaligned(_) => 0,
            Self::InstructionAccessFault(_) => 1,
            Self::IllegalInstruction(_) => 2,
            Self::BreakPoint => 3,
            Self::LoadAddressMisaligned(_) => 4,
            Self::LoadAccessFault(_) => 5,
            Self::StoreAddressMisaligned(_) => 6,
            Self::StoreAccessFault(_) => 7,
            Self::EnvironmentCallU => 8,
            Self::EnvironmentCallM => 11,
            Self::TimerInterrupt => 0x8000_0007,
        }
    }
}
