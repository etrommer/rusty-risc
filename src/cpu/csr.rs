use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Primitive)]
pub enum ArchCSRs {
    Mvendorid = 0xf11,
    Marchid = 0xf12,
    Mimpid = 0xf13,
    Mhartid = 0xf14,

    Mstatus = 0x300,
    Misa = 0x301,
    Mie = 0x304,
    Mtvec = 0x305,

    Mscratch = 0x340,
    Mepc = 0x341,
    Mcause = 0x342,
    Mtval = 0x343,
    Mip = 0x344,
}

const ARCH_CSRS_ITERABLE: [ArchCSRs; 13] = [
    ArchCSRs::Mvendorid,
    ArchCSRs::Marchid,
    ArchCSRs::Mimpid,
    ArchCSRs::Mhartid,
    ArchCSRs::Mstatus,
    ArchCSRs::Misa,
    ArchCSRs::Mie,
    ArchCSRs::Mtvec,
    ArchCSRs::Mscratch,
    ArchCSRs::Mepc,
    ArchCSRs::Mcause,
    ArchCSRs::Mtval,
    ArchCSRs::Mip,
];

struct RegisterU32 {
    value: u32,
    writable: bool,
}

pub struct CSRFile {
    csrs: HashMap<ArchCSRs, RegisterU32>,
}

impl CSRFile {
    pub fn new() -> Self {
        let mut map: HashMap<ArchCSRs, RegisterU32> = HashMap::new();
        for e in ARCH_CSRS_ITERABLE.iter() {
            let writable = match e {
                ArchCSRs::Mvendorid => false,
                ArchCSRs::Marchid => false,
                ArchCSRs::Mimpid => false,
                ArchCSRs::Mhartid => false,
                _ => true,
            };
            let initial_value = match e {
                ArchCSRs::Mvendorid => 0xff0f_f0ff,
                ArchCSRs::Misa => 0x4040_1101, // (XLEN=32, IMA+X)
                _ => 0x0000_0000,
            };
            map.insert(
                e.clone(),
                RegisterU32 {
                    value: initial_value,
                    writable: writable,
                },
            );
        }
        Self { csrs: map }
    }

    pub fn write(&mut self, addr: i32, value: i32) {
        if let Some(register) = ArchCSRs::from_i32(addr) {
            let csr = self.csrs.get_mut(&register).unwrap();
            if csr.writable {
                csr.value = value as u32;
            }
        }
    }

    pub fn read(&self, addr: i32) -> i32 {
        if let Some(register) = ArchCSRs::from_i32(addr) {
            return self.csrs.get(&register).unwrap().value as i32;
        }
        0
    }
}
