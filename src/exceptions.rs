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
