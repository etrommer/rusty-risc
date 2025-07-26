use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::{collections::HashMap, convert::TryInto, path::is_separator};

#[derive(Debug, Clone, Eq, Hash, PartialEq, Primitive)]
enum ArchCSRs {
    mvendorid = 0xf11,
    marchid = 0xf12,
    mimpid = 0xf13,
    mhartid = 0xf14,

    mstatus = 0x300,
    misa = 0x301,
    mie = 0x304,
    mtvec = 0x305,

    mscratch = 0x340,
    mepc = 0x341,
    mcause = 0x342,
    mtval = 0x343,
    mip = 0x344,
}

const ARCH_CSRS_ITERABLE: [ArchCSRs; 13] = [
    ArchCSRs::mvendorid,
    ArchCSRs::marchid,
    ArchCSRs::mimpid,
    ArchCSRs::mhartid,
    ArchCSRs::mstatus,
    ArchCSRs::misa,
    ArchCSRs::mie,
    ArchCSRs::mtvec,
    ArchCSRs::mscratch,
    ArchCSRs::mepc,
    ArchCSRs::mcause,
    ArchCSRs::mtval,
    ArchCSRs::mip,
];

struct RegisterU32 {
    value: u32,
    writable: bool,
}

pub struct CSRs {
    csrs: HashMap<ArchCSRs, RegisterU32>,
}

impl CSRs {
    pub fn new() -> Self {
        let mut map: HashMap<ArchCSRs, RegisterU32> = HashMap::new();
        for e in ARCH_CSRS_ITERABLE.iter() {
            let writable = match e {
                ArchCSRs::mvendorid => false,
                ArchCSRs::marchid => false,
                ArchCSRs::mimpid => false,
                ArchCSRs::mhartid => false,
                _ => true,
            };
            let initial_value = match e {
                ArchCSRs::mvendorid => 0xff0f_f0ff,
                ArchCSRs::misa => 0x4040_1101, // (XLEN=32, IMA+X)
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
            let mut csr = self.csrs.get_mut(&register).unwrap();
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
