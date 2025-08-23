use num_traits::FromPrimitive;
use tracing::info;

use super::instructions::{IInstruction, Instruction, RInstruction, SBInstruction, UJInstruction};
use super::Cpu;
use crate::bus::{BusDevice, BusError};
use crate::cpu::csr::ArchCSRs;
use crate::cpu::ExecMode;
use crate::trap::RVException;

fn handle_load_error(e: BusError) -> RVException {
    match e {
        BusError::AddressMisaligned(addr) => RVException::LoadAddressMisaligned(addr),
        BusError::AddressUnmapped(addr) => RVException::LoadAccessFault(addr),
    }
}

fn handle_store_error(e: BusError) -> RVException {
    match e {
        BusError::AddressMisaligned(addr) => RVException::StoreAddressMisaligned(addr),
        BusError::AddressUnmapped(addr) => RVException::StoreAccessFault(addr),
    }
}

fn exec_i(
    cpu: &mut Cpu,
    rs1: usize,
    rd: usize,
    imm: i32,
    inst: IInstruction,
) -> Result<(), RVException> {
    // Load rs1 contents
    let rs1_data = cpu.regfile.read(rs1);

    // Handle all instructions that write back to rd
    if let Some(result) = match inst {
        // Arithmetic
        IInstruction::addi => Some(rs1_data.wrapping_add(imm)),
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

        // Zicsr Instructions
        IInstruction::csrrw => {
            let register_data = cpu.csrfile.read(imm);
            cpu.csrfile.write(imm, rs1_data);
            Some(register_data)
        }
        IInstruction::csrrs => {
            let register_data = cpu.csrfile.read(imm);
            cpu.csrfile.write(imm, register_data | rs1_data);
            Some(register_data)
        }
        IInstruction::csrrc => {
            let register_data = cpu.csrfile.read(imm);
            cpu.csrfile.write(imm, register_data & !rs1_data);
            Some(register_data)
        }
        IInstruction::csrrwi => {
            let register_data = cpu.csrfile.read(imm);
            let uimm = rs1 as u32;
            cpu.csrfile.write(imm, uimm as i32);
            Some(register_data)
        }
        IInstruction::csrrsi => {
            let register_data = cpu.csrfile.read(imm);
            let uimm = rs1 as u32;
            cpu.csrfile.write(imm, register_data | uimm as i32);
            Some(register_data)
        }
        IInstruction::csrrci => {
            let register_data = cpu.csrfile.read(imm);
            let uimm = rs1 as u32;
            cpu.csrfile.write(imm, register_data & !(uimm as i32));
            Some(register_data)
        }

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
            IInstruction::ecall => match cpu.mode {
                ExecMode::MACHINE => Err(RVException::EnvironmentCallM),
                ExecMode::USER => Err(RVException::EnvironmentCallU),
            },

            // Sequential execution anyway, so no
            // need to fence anything
            IInstruction::fence => Ok(()),
            IInstruction::fencei => Ok(()),

            IInstruction::mret => {
                // Re-enable interrupts
                cpu.csrfile.enable_irq();

                // Restore PC from mepc
                cpu.pc = cpu.csrfile.read(ArchCSRs::mepc as i32) as u32 as usize - 4;

                cpu.mode = ExecMode::from_u32(cpu.csrfile.get_mpp()).unwrap();
                cpu.csrfile.set_mpp(&(ExecMode::MACHINE as u32));
                info!(
                    "Returning from trap to mode {:?}, mstatus: {:#010x}, PC: {:#010x}",
                    cpu.mode,
                    cpu.csrfile.read(ArchCSRs::mstatus as i32),
                    cpu.pc
                );

                Ok(())
            }
            IInstruction::sret => {
                panic!("Supervisor mode not implemented yet");
            }
            IInstruction::wfi => {
                // Ignore sleep for now
                Ok(())
            }

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

    // Generic closure for atomic logic instructions
    // to make the implementation less verbose.
    let mut amo_logic = |operation: fn(i32, i32) -> i32| -> Result<i32, RVException> {
        let mem_value = cpu
            .bus
            .load::<i32>(rs1_data as u32 as usize)
            .map_err(|e| handle_load_error(e))?;
        let result = operation(mem_value, rs2_data);
        cpu.bus
            .store::<i32>(rs1_data as u32 as usize, result)
            .map_err(|e| handle_store_error(e))?;

        Ok(mem_value)
    };

    let result = match inst {
        RInstruction::add => rs1_data.wrapping_add(rs2_data),
        RInstruction::sub => rs1_data.wrapping_sub(rs2_data),
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

        // A-Extension instructions
        RInstruction::amoAddW => amo_logic(|a, b| a.wrapping_add(b))?,
        RInstruction::amoAndW => amo_logic(|a, b| a & b)?,
        RInstruction::amoOrW => amo_logic(|a, b| a | b)?,
        RInstruction::amoXorW => amo_logic(|a, b| a ^ b)?,
        RInstruction::amoMaxW => amo_logic(|a, b| a.max(b))?,
        RInstruction::amoMinW => amo_logic(|a, b| a.min(b))?,
        RInstruction::amoMaxUW => amo_logic(|a, b| (a as u32).max(b as u32) as i32)?,
        RInstruction::amoMinUW => amo_logic(|a, b| (a as u32).min(b as u32) as i32)?,
        RInstruction::amoSwapW => amo_logic(|_, b| b)?,
        RInstruction::lrw => {
            let addr = rs1_data as u32 as usize;
            let mem_value = cpu
                .bus
                .load::<i32>(addr)
                .map_err(|e| handle_load_error(e))?;
            cpu.amoreserved.insert(addr);
            mem_value
        }
        RInstruction::scw => {
            let addr = rs1_data as u32 as usize;
            if cpu.amoreserved.remove(&addr) {
                cpu.bus
                    .store::<i32>(addr, rs2_data)
                    .map_err(|e| handle_store_error(e))?;
                0
            } else {
                1
            }
        }

        // M-Extension instructions
        RInstruction::mul => (rs1_data as i64 * rs2_data as i64) as i32,
        RInstruction::mulh => ((rs1_data as i64 * rs2_data as i64) >> 32) as i32,
        RInstruction::mulhu => ((rs1_data as u32 as u64 * rs2_data as u32 as u64) >> 32) as i32,
        RInstruction::mulhsu => ((rs1_data as i64 * rs2_data as u32 as i64) >> 32) as i32,
        RInstruction::div => {
            if rs2_data == 0 {
                -1
            } else {
                rs1_data.wrapping_div(rs2_data)
            }
        }
        RInstruction::divu => {
            if rs2_data == 0 {
                -1
            } else {
                (rs1_data as u32).wrapping_div(rs2_data as u32) as i32
            }
        }
        RInstruction::rem => {
            if rs2_data == 0 {
                rs1_data
            } else {
                rs1_data.wrapping_rem(rs2_data)
            }
        }
        RInstruction::remu => {
            if rs2_data == 0 {
                rs1_data
            } else {
                (rs1_data as u32).wrapping_rem(rs2_data as u32) as i32
            }
        }
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
        UJInstruction::auipc => old_pc.wrapping_add(imm << 12),
        UJInstruction::lui => imm << 12,
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
