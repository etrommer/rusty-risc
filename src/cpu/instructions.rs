use core::fmt;
use enum_primitive_derive::Primitive;

#[derive(Debug, Clone, PartialEq, Primitive)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    ARITH_REG = 0b_011_0011,
    ARITH_IMM = 0b_001_0011,
    LOAD = 0b_000_0011,
    STORE = 0b_010_0011,
    BRANCH = 0b_110_0011,
    JAL = 0b_110_1111,
    JALR = 0b_110_0111,
    LUI = 0b_011_0111,
    AUIPC = 0b_001_0111,
    FENCE = 0b_000_1111,
    SYSTEM = 0b_111_0011,
    ATOMIC = 0b010_1111,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum RInstruction {
    add,
    sub,
    xor,
    or,
    and,

    sll,
    srl,
    sra,
    slt,
    sltu,

    mul,
    mulh,
    mulhsu,
    mulhu,
    div,
    divu,
    rem,
    remu,

    lrw,
    scw,
    amoSwapW,
    amoAddW,
    amoXorW,
    amoAndW,
    amoOrW,
    amoMinW,
    amoMaxW,
    amoMinUW,
    amoMaxUW,
}

impl RInstruction {
    pub fn new(opcode: &Opcode, funct3: &u32, funct7: &u32) -> Option<Self> {
        if *opcode == Opcode::ATOMIC && *funct3 == 0b010 {
            let funct = funct7 >> 2;
            return match funct {
                0b00010 => Some(RInstruction::lrw),
                0b00011 => Some(RInstruction::scw),

                0b00001 => Some(RInstruction::amoSwapW),
                0b00000 => Some(RInstruction::amoAddW),

                0b00100 => Some(RInstruction::amoXorW),
                0b01100 => Some(RInstruction::amoAndW),
                0b01000 => Some(RInstruction::amoOrW),
                0b10000 => Some(RInstruction::amoMinW),
                0b10100 => Some(RInstruction::amoMaxW),

                0b11000 => Some(RInstruction::amoMinUW),
                0b11100 => Some(RInstruction::amoMaxUW),

                _ => None,
            };
        }
        match (funct3, funct7) {
            (0x0, 0x0) => Some(RInstruction::add),
            (0x0, 0x20) => Some(RInstruction::sub),

            (0x4, 0x0) => Some(RInstruction::xor),
            (0x6, 0x0) => Some(RInstruction::or),
            (0x7, 0x0) => Some(RInstruction::and),

            (0x1, 0x0) => Some(RInstruction::sll),
            (0x5, 0x0) => Some(RInstruction::srl),
            (0x5, 0x20) => Some(RInstruction::sra),

            (0x2, 0x0) => Some(RInstruction::slt),
            (0x3, 0x0) => Some(RInstruction::sltu),

            (0b000, 1) => Some(RInstruction::mul),
            (0b001, 1) => Some(RInstruction::mulh),
            (0b010, 1) => Some(RInstruction::mulhsu),
            (0b011, 1) => Some(RInstruction::mulhu),
            (0b100, 1) => Some(RInstruction::div),
            (0b101, 1) => Some(RInstruction::divu),
            (0b110, 1) => Some(RInstruction::rem),
            (0b111, 1) => Some(RInstruction::remu),

            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum IInstruction {
    addi,
    xori,
    ori,
    andi,

    slli,
    srli,
    srai,
    slti,
    sltiu,

    lb,
    lh,
    lw,
    lbu,
    lhu,

    jalr,

    ecall,
    ebreak,
    fence,
    fencei,

    csrrw,
    csrrs,
    csrrc,
    csrrwi,
    csrrsi,
    csrrci,
}

impl IInstruction {
    pub fn new(opcode: &Opcode, funct3: &u32, imm: &i32) -> Option<Self> {
        if *opcode == Opcode::SYSTEM && *funct3 == 0 {
            if *imm == 0 {
                return Some(IInstruction::ecall);
            }
            if *imm == 1 {
                return Some(IInstruction::ebreak);
            }
        }

        let upper_imm = (*imm as u32) >> 5;
        match (opcode, funct3, upper_imm) {
            (Opcode::ARITH_IMM, 0b001, 0x00) => Some(IInstruction::slli),
            (Opcode::ARITH_IMM, 0b101, 0x00) => Some(IInstruction::srli),
            (Opcode::ARITH_IMM, 0b101, 0x20) => Some(IInstruction::srai),

            (Opcode::ARITH_IMM, 0x0, _) => Some(IInstruction::addi),
            (Opcode::ARITH_IMM, 0x4, _) => Some(IInstruction::xori),
            (Opcode::ARITH_IMM, 0x6, _) => Some(IInstruction::ori),
            (Opcode::ARITH_IMM, 0x7, _) => Some(IInstruction::andi),

            (Opcode::ARITH_IMM, 0x2, _) => Some(IInstruction::slti),
            (Opcode::ARITH_IMM, 0x3, _) => Some(IInstruction::sltiu),

            (Opcode::LOAD, 0x0, _) => Some(IInstruction::lb),
            (Opcode::LOAD, 0x1, _) => Some(IInstruction::lh),
            (Opcode::LOAD, 0x2, _) => Some(IInstruction::lw),
            (Opcode::LOAD, 0x4, _) => Some(IInstruction::lbu),
            (Opcode::LOAD, 0x5, _) => Some(IInstruction::lhu),

            (Opcode::JALR, 0x0, _) => Some(IInstruction::jalr),

            (Opcode::FENCE, 0x0, _) => Some(IInstruction::fence),
            (Opcode::FENCE, 0x1, _) => Some(IInstruction::fencei),

            (Opcode::SYSTEM, 0b001, _) => Some(IInstruction::csrrw),
            (Opcode::SYSTEM, 0b010, _) => Some(IInstruction::csrrs),
            (Opcode::SYSTEM, 0b011, _) => Some(IInstruction::csrrc),
            (Opcode::SYSTEM, 0b101, _) => Some(IInstruction::csrrwi),
            (Opcode::SYSTEM, 0b110, _) => Some(IInstruction::csrrsi),
            (Opcode::SYSTEM, 0b111, _) => Some(IInstruction::csrrci),

            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum UJInstruction {
    jal,
    lui,
    auipc,
}

impl UJInstruction {
    pub fn new(opcode: Opcode) -> Self {
        match opcode {
            Opcode::JAL => UJInstruction::jal,
            Opcode::LUI => UJInstruction::lui,
            Opcode::AUIPC => UJInstruction::auipc,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum SBInstruction {
    sb,
    sh,
    sw,
    beq,
    bne,
    blt,
    bge,
    bltu,
    bgeu,
}

impl SBInstruction {
    pub fn new(opcode: &Opcode, funct3: &u32) -> Option<Self> {
        match (opcode, funct3) {
            (Opcode::STORE, 0x0) => Some(SBInstruction::sb),
            (Opcode::STORE, 0x1) => Some(SBInstruction::sh),
            (Opcode::STORE, 0x2) => Some(SBInstruction::sw),

            (Opcode::BRANCH, 0x0) => Some(SBInstruction::beq),
            (Opcode::BRANCH, 0x1) => Some(SBInstruction::bne),
            (Opcode::BRANCH, 0x4) => Some(SBInstruction::blt),
            (Opcode::BRANCH, 0x5) => Some(SBInstruction::bge),
            (Opcode::BRANCH, 0x6) => Some(SBInstruction::bltu),
            (Opcode::BRANCH, 0x7) => Some(SBInstruction::bgeu),

            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    RType {
        rd: usize,
        rs1: usize,
        rs2: usize,
        inst: RInstruction,
    },
    IType {
        imm: i32,
        rd: usize,
        rs1: usize,
        inst: IInstruction,
    },
    SBType {
        imm: i32,
        rs1: usize,
        rs2: usize,
        inst: SBInstruction,
    },
    UJType {
        imm: i32,
        rd: usize,
        inst: UJInstruction,
    },
}

fn pretty_register(num: &usize) -> &str {
    const REG_NAMES: [&str; 32] = [
        "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
        "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
        "t5", "t6",
    ];
    REG_NAMES[*num]
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RType { rd, rs1, rs2, inst } => {
                write!(
                    f,
                    "{:?} - {}, {}, {} (R)",
                    inst,
                    pretty_register(rd),
                    pretty_register(rs1),
                    pretty_register(rs2)
                )
            }
            Self::IType { imm, rd, rs1, inst } => {
                write!(
                    f,
                    "{:?} - {}, {}, {:#x} (I) ",
                    inst,
                    pretty_register(rd),
                    pretty_register(rs1),
                    imm
                )
            }
            Self::SBType {
                imm,
                rs1,
                rs2,
                inst,
            } => {
                write!(
                    f,
                    "{:?} - {}, {}, {:#x} (S/B)",
                    inst,
                    pretty_register(rs1),
                    pretty_register(rs2),
                    imm
                )
            }
            Self::UJType { imm, rd, inst } => {
                write!(f, "{:?} - {}, {:#x} (U/J)", inst, pretty_register(rd), imm)
            }
        }
    }
}
