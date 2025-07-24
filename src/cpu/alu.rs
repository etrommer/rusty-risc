use super::instructions::{IInstruction, Instruction, RInstruction, SBInstruction, UJInstruction};
use super::Cpu;
use crate::bus::{BusDevice, BusError};
use crate::exceptions::RVException;

fn exec_i(
    cpu: &mut Cpu,
    rs1: usize,
    rd: usize,
    imm: i32,
    inst: IInstruction,
) -> Result<(), RVException> {
    // Load rs1 contents
    let rs1_data = cpu.regfile.read(rs1);

    fn handle_load_error(e: BusError) -> RVException {
        match e {
            BusError::AddressMisaligned(addr) => RVException::LoadAddressMisaligned(addr),
            BusError::AddressUnmapped(addr) => RVException::LoadAccessFault(addr),
        }
    }

    // Handle all instructions that write back to rd
    if let Some(result) = match inst {
        // Arithmetic
        IInstruction::addi => Some(rs1_data + imm),
        IInstruction::xori => Some(rs1_data ^ imm),
        IInstruction::ori => Some(rs1_data | imm),
        IInstruction::andi => Some(rs1_data & imm),
        IInstruction::slli => Some(rs1_data << (imm & 0x1f)),
        IInstruction::srli => Some(((rs1_data as u32) >> (imm & 0x1f)) as i32),
        IInstruction::srai => Some(rs1_data >> (imm & 0x1f)),
        IInstruction::slti => {
            if rs1_data < imm {
                Some(1)
            } else {
                Some(0)
            }
        }
        IInstruction::sltiu => {
            if (rs1_data as u32) < (imm as u32) {
                Some(1)
            } else {
                Some(0)
            }
        }

        // Load
        IInstruction::lb => Some(
            cpu.bus
                .load::<i8>((rs1_data + imm) as u32 as usize)
                .map_err(|e| handle_load_error(e))? as i32,
        ),
        IInstruction::lh => Some(
            cpu.bus
                .load::<i16>((rs1_data + imm) as u32 as usize)
                .map_err(|e| handle_load_error(e))? as i32,
        ),
        IInstruction::lw => Some(
            cpu.bus
                .load::<i32>((rs1_data + imm) as u32 as usize)
                .map_err(|e| handle_load_error(e))? as i32,
        ),
        IInstruction::lbu => Some(
            cpu.bus
                .load::<u8>((rs1_data + imm) as u32 as usize)
                .map_err(|e| handle_load_error(e))? as i32,
        ),
        IInstruction::lhu => Some(
            cpu.bus
                .load::<u16>((rs1_data + imm) as u32 as usize)
                .map_err(|e| handle_load_error(e))? as i32,
        ),

        // Jump
        IInstruction::jalr => {
            let old_pc = cpu.pc as i32;
            cpu.pc = (rs1_data + imm - 4) as u32 as usize;
            Some(old_pc + 4)
        }

        // Handle Ecall and Ebreak instructions separately
        _ => None,
    } {
        // Write result back to rd
        cpu.regfile.write(rd, result);
        return Ok(());
    } else {
        match inst {
            IInstruction::ebreak => Err(RVException::BreakPoint),
            IInstruction::ecall => Err(RVException::EnvironmentCall),

            // Sequential execution anyway, so no
            // need to fence anything
            IInstruction::fence => Ok(()),
            IInstruction::fencei => Ok(()),

            IInstruction::csrrc => todo!(),
            IInstruction::csrrs => todo!(),
            IInstruction::csrrw => todo!(),
            IInstruction::csrrci => todo!(),
            IInstruction::csrrsi => todo!(),
            IInstruction::csrrwi => todo!(),

            _ => panic!("Unimplemented instruction: {:?}", inst),
        }
    }
}

fn exec_r(
    cpu: &mut Cpu,
    rd: usize,
    rs1: usize,
    rs2: usize,
    inst: RInstruction,
) -> Result<(), RVException> {
    let rs1_data = cpu.regfile.read(rs1);
    let rs2_data = cpu.regfile.read(rs2);

    let result = match inst {
        RInstruction::add => rs1_data + rs2_data,
        RInstruction::sub => rs1_data - rs2_data,
        RInstruction::xor => rs1_data ^ rs2_data,
        RInstruction::or => rs1_data | rs2_data,
        RInstruction::and => rs1_data & rs2_data,
        RInstruction::sll => rs1_data << rs2_data,
        RInstruction::srl => ((rs1_data as u32) >> rs2_data) as i32,
        RInstruction::sra => rs1_data >> rs2_data,
        RInstruction::slt => {
            if rs1_data < rs2_data {
                1
            } else {
                0
            }
        }
        RInstruction::sltu => {
            if (rs1_data as u32) < (rs2_data as u32) {
                1
            } else {
                0
            }
        }

        // Atomics & M-Instructions;
        _ => todo!(),
    };
    cpu.regfile.write(rd, result);
    Ok(())
}

fn exec_s_b(
    cpu: &mut Cpu,
    imm: i32,
    rs1: usize,
    rs2: usize,
    inst: SBInstruction,
) -> Result<(), RVException> {
    let rs1_data = cpu.regfile.read(rs1);
    let rs2_data = cpu.regfile.read(rs2);
    if let Some(result) = match inst {
        // Stores
        SBInstruction::sb => Some(
            cpu.bus
                .store::<i8>((rs1_data + imm) as u32 as usize, rs2_data as i8),
        ),
        SBInstruction::sh => Some(
            cpu.bus
                .store::<i16>((rs1_data + imm) as u32 as usize, rs2_data as i16),
        ),
        SBInstruction::sw => Some(
            cpu.bus
                .store::<i32>((rs1_data + imm) as u32 as usize, rs2_data),
        ),
        _ => None,
    } {
        return result.map_err(|e| match e {
            BusError::AddressMisaligned(addr) => RVException::StoreAddressMisaligned(addr),
            BusError::AddressUnmapped(addr) => RVException::StoreAccessFault(addr),
        });
    } else {
        // Conditional jumps
        let jump_taken = match inst {
            SBInstruction::beq => rs1_data == rs2_data,
            SBInstruction::bne => rs1_data != rs2_data,
            SBInstruction::blt => rs1_data < rs2_data,
            SBInstruction::bge => rs1_data >= rs2_data,
            SBInstruction::bltu => (rs1_data as u32) < (rs2_data as u32),
            SBInstruction::bgeu => (rs1_data as u32) >= (rs2_data as u32),
            _ => unreachable!(),
        };
        if jump_taken {
            // Set PC to instruction *before* jump target
            // PC is incremented unconditionally by 4 after each instruction
            let jump_target = (cpu.pc as isize) + (imm as isize) - 4;
            cpu.pc = jump_target as usize;
        }
    }
    Ok(())
}

fn exec_u_j(cpu: &mut Cpu, imm: i32, rd: usize, inst: UJInstruction) -> Result<(), RVException> {
    let old_pc = cpu.pc as i32;
    let result = match inst {
        UJInstruction::auipc => old_pc + imm,
        UJInstruction::lui => imm,
        UJInstruction::jal => {
            // Set PC to instruction *before* jump target
            // PC is incremented unconditionally by 4 after each instruction
            // cpu.pc += imm as usize - 4;
            let old_pc = cpu.pc as isize;
            cpu.pc = (old_pc + (imm as isize) - 4) as usize;

            (old_pc + 4) as i32
        }
    };
    cpu.regfile.write(rd, result);
    Ok(())
}

pub fn exec(cpu: &mut Cpu, instruction: Instruction) -> Result<(), RVException> {
    match instruction {
        Instruction::IType { rd, rs1, imm, inst } => exec_i(cpu, rs1, rd, imm, inst),
        Instruction::RType { rd, rs1, rs2, inst } => exec_r(cpu, rd, rs1, rs2, inst),
        Instruction::SBType {
            imm,
            rs1,
            rs2,
            inst,
        } => exec_s_b(cpu, imm, rs1, rs2, inst),
        Instruction::UJType { imm, rd, inst } => exec_u_j(cpu, imm, rd, inst),
    }
}
