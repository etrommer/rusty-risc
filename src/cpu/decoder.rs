use num_traits::FromPrimitive;

use super::instructions::{IInstruction, RInstruction, SBInstruction, UJInstruction};
use super::instructions::{Instruction, Opcode};
use crate::trap::RVException;

// Decode instruction from raw integer
pub fn decode(raw: &u32) -> Result<Instruction, RVException> {
    // Extract registers
    let rd = |raw: &u32| ((raw >> 7) & 0x1f) as usize;
    let rs1 = |raw: &u32| ((raw >> 15) & 0x1f) as usize;
    let rs2 = |raw: &u32| ((raw >> 20) & 0x1f) as usize;

    // Extract funct codes
    let f3 = |raw: &u32| (raw >> 12) & 0b111;
    let f7 = |raw: &u32| (raw >> 25);

    // Extract immediates
    let imm_i = |raw: &u32| ((*raw as i32) >> 20);
    let imm_b = |raw: &u32| {
        ((*raw as i32) >> 31) << 12
            | (((raw >> 7) & 0b1) << 11) as i32
            | (((raw >> 25) & 0b111111) << 5) as i32
            | (((raw >> 8) & 0b1111) << 1) as i32
    };
    let imm_s = |raw: &u32| (((*raw as i32) >> 25) << 5) as i32 | ((raw >> 7) & 0x1f) as i32;
    let imm_j = |raw: &u32| {
        (((*raw as i32) >> 31) << 20) as i32
            | (raw & (0xff << 12)) as i32
            | (((raw >> 20) & 1) << 11) as i32
            | (((raw >> 21) & 0x3ff) << 1) as i32
    };
    let imm_u = |raw: &u32| (*raw as i32) >> 12;

    // Extract Opcode
    let raw_opcode = raw & 0x7f;

    // Turn raw opcode into a nice enum
    if let Some(opcode) = Opcode::from_u32(raw_opcode) {
        match opcode {
            // R-Type format
            Opcode::ARITH_REG | Opcode::ATOMIC => {
                let funct3 = f3(raw);
                let funct7 = f7(raw);
                if let Some(inst) = RInstruction::new(&opcode, &funct3, &funct7) {
                    Ok(Instruction::RType {
                        rd: rd(raw),
                        rs1: rs1(raw),
                        rs2: rs2(raw),
                        inst,
                    })
                } else {
                    Err(RVException::IllegalInstruction(*raw))
                }
            }
            // I-Type format
            Opcode::ARITH_IMM | Opcode::LOAD | Opcode::JALR | Opcode::SYSTEM | Opcode::FENCE => {
                let imm = imm_i(raw);
                let funct3 = f3(raw);
                if let Some(inst) = IInstruction::new(&opcode, &funct3, &imm) {
                    Ok(Instruction::IType {
                        rd: rd(raw),
                        rs1: rs1(raw),
                        imm,
                        inst,
                    })
                } else {
                    Err(RVException::IllegalInstruction(*raw))
                }
            }
            // S/B-Type format
            Opcode::BRANCH => {
                let funct3 = f3(raw);
                if let Some(inst) = SBInstruction::new(&opcode, &funct3) {
                    Ok(Instruction::SBType {
                        imm: imm_b(raw),
                        rs1: rs1(raw),
                        rs2: rs2(raw),
                        inst,
                    })
                } else {
                    Err(RVException::IllegalInstruction(*raw))
                }
            }
            Opcode::STORE => {
                let funct3 = f3(raw);
                if let Some(inst) = SBInstruction::new(&opcode, &funct3) {
                    Ok(Instruction::SBType {
                        imm: imm_s(raw),
                        rs1: rs1(raw),
                        rs2: rs2(raw),
                        inst,
                    })
                } else {
                    Err(RVException::IllegalInstruction(*raw))
                }
            }
            Opcode::JAL => Ok(Instruction::UJType {
                imm: imm_j(raw),
                rd: rd(raw),
                inst: UJInstruction::new(opcode),
            }),
            Opcode::LUI | Opcode::AUIPC => Ok(Instruction::UJType {
                imm: imm_u(raw),
                rd: rd(raw),
                inst: UJInstruction::new(opcode),
            }),
        }
    } else {
        Err(RVException::IllegalInstruction(*raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtype() {
        assert_eq!(
            decode(&0x007302b3),
            Ok(Instruction::RType {
                rd: 5,
                rs1: 6,
                rs2: 7,
                inst: RInstruction::add
            })
        );
    }

    #[test]
    fn test_itype() {
        assert_eq!(
            decode(&0xfef2c113),
            Ok(Instruction::IType {
                imm: -17,
                rd: 2,
                rs1: 5,
                inst: IInstruction::xori
            })
        );
        assert_eq!(
            decode(&0x1f51513),
            Ok(Instruction::IType {
                imm: 31,
                rd: 10,
                rs1: 10,
                inst: IInstruction::slli
            })
        )
    }
    #[test]
    fn test_utype() {
        assert_eq!(
            decode(&0x80000117),
            Ok(Instruction::UJType {
                imm: -524288,
                rd: 2,
                inst: UJInstruction::auipc
            })
        );
        assert_eq!(
            decode(&0xaaaaa0b7),
            Ok(Instruction::UJType {
                imm: -349526,
                rd: 1,
                inst: UJInstruction::lui
            })
        )
    }
    #[test]
    fn test_stype() {
        assert_eq!(
            decode(&0x81d12023),
            Ok(Instruction::SBType {
                imm: -2048,
                rs2: 29,
                rs1: 2,
                inst: SBInstruction::sw
            })
        )
    }
    #[test]
    fn test_jtype() {
        assert_eq!(
            decode(&0x2abaa06f),
            Ok(Instruction::UJType {
                rd: 0,
                imm: 699050,
                inst: UJInstruction::jal
            })
        );
        assert_eq!(
            decode(&0xd545506f),
            Ok(Instruction::UJType {
                rd: 0,
                imm: -699052,
                inst: UJInstruction::jal
            })
        );
        assert_eq!(
            decode(&0x8000006f),
            Ok(Instruction::UJType {
                rd: 0,
                imm: -0x100000,
                inst: UJInstruction::jal
            })
        )
    }
    #[test]
    fn test_btype() {
        assert_eq!(
            decode(&0xfad10ee3),
            Ok(Instruction::SBType {
                imm: -68,
                rs2: 13,
                rs1: 2,
                inst: SBInstruction::beq
            })
        );
        assert_eq!(
            decode(&0x7ff00fe3),
            Ok(Instruction::SBType {
                imm: 4094,
                rs2: 31,
                rs1: 0,
                inst: SBInstruction::beq
            })
        );
        assert_eq!(
            decode(&0x8113c063),
            Ok(Instruction::SBType {
                imm: -4096,
                rs2: 17,
                rs1: 7,
                inst: SBInstruction::blt
            })
        );
    }
}
