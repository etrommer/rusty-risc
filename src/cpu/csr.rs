use enum_primitive_derive::Primitive;

#[derive(Debug, Clone, PartialEq, Primitive)]
pub enum CSR {
    mscratch = 0x340,
    mtvec = 0x305,
    mie = 0x304,
    mip = 0x344,
    mepc = 0x341,
    mstatus = 0x300,
    mcause = 0x342,
    mtval = 0x343,
    mvendorid = 0xf11,
    misa = 0x301,
}
