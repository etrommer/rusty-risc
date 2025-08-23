use crate::{cpu::MMIORegister, trap::RVException};
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use tracing::warn;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Primitive)]
#[allow(non_camel_case_types)]
pub enum ArchCSRs {
    mvendorid = 0xf11,
    marchid = 0xf12,
    mimpid = 0xf13,
    mhartid = 0xf14,

    rdcycle = 0xc00,

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

const ARCH_CSRS_ITERABLE: [ArchCSRs; 14] = [
    ArchCSRs::mvendorid,
    ArchCSRs::marchid,
    ArchCSRs::mimpid,
    ArchCSRs::mhartid,
    ArchCSRs::rdcycle,
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

pub struct CSRFile {
    csrs: HashMap<ArchCSRs, MMIORegister>,
}

impl CSRFile {
    pub fn new() -> Self {
        let mut map: HashMap<ArchCSRs, MMIORegister> = HashMap::new();
        for e in ARCH_CSRS_ITERABLE.iter() {
            let writable = match e {
                ArchCSRs::mvendorid => false,
                ArchCSRs::marchid => false,
                ArchCSRs::mimpid => false,
                ArchCSRs::mhartid => false,
                ArchCSRs::rdcycle => false,
                _ => true,
            };
            let initial_value = match e {
                ArchCSRs::mvendorid => 0xff0f_f0ff,
                ArchCSRs::misa => 0x4040_1101, // (XLEN=32, IMA+X)
                _ => 0x0000_0000,
            };
            map.insert(
                e.clone(),
                MMIORegister {
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

    pub fn count_cycle(&mut self) {
        if let Some(rdcycle) = self.csrs.get_mut(&ArchCSRs::rdcycle) {
            rdcycle.value = rdcycle.value.wrapping_add(1);
        }
    }

    pub fn disable_irq(&mut self) {
        const MSTATUS_MIE: u32 = 1 << 3;
        const MSTATUS_MPIE: u32 = 1 << 7;

        let mstatus = self.csrs.get_mut(&ArchCSRs::mstatus).unwrap();
        // Save MIE bit to MPIE
        if (mstatus.value & MSTATUS_MIE) != 0 {
            mstatus.value |= MSTATUS_MPIE;
        } else {
            mstatus.value &= MSTATUS_MPIE;
        }
        // Clear MIE to disable interrupts
        mstatus.value &= !MSTATUS_MIE;
    }

    pub fn get_mpp(&self) -> u32 {
        const MSTATUS_MPP: u32 = 0b11 << 11; // MPP bits in mstatus CSR
        let mstatus = self.csrs.get(&ArchCSRs::mstatus).unwrap();
        (mstatus.value & MSTATUS_MPP) >> 11
    }

    pub fn set_mpp(&mut self, mpp: &u32) {
        const MSTATUS_MPP: u32 = 0b11 << 11; // MPP bits in mstatus CSR
        let mstatus = self.csrs.get_mut(&ArchCSRs::mstatus).unwrap();
        // Clear MPP bits and set new value
        mstatus.value = (mstatus.value & !MSTATUS_MPP) | ((mpp & 0b11) << 11);
    }

    pub fn enable_irq(&mut self) {
        const MSTATUS_MIE: u32 = 1 << 3; // MIE bit in mstatus CSR
        const MSTATUS_MPIE: u32 = 1 << 7; // MPIE bit in mstatus CSR

        let mstatus = self.csrs.get_mut(&ArchCSRs::mstatus).unwrap();
        // Restore previous MIE state from MPIE
        if (mstatus.value & MSTATUS_MPIE) != 0 {
            mstatus.value |= MSTATUS_MIE;
        } else {
            mstatus.value &= !MSTATUS_MIE;
        }
        // Set MPIE bit
        mstatus.value |= MSTATUS_MPIE;
    }

    pub fn mtimer_interrupt(&self) -> Result<(), RVException> {
        const MIE_MTIE: u32 = 1 << 7;
        const MSTATUS_MIE: u32 = 1 << 3;
        const MIP_MTIP: u32 = 1 << 7;

        let mie = self.csrs.get(&ArchCSRs::mie).unwrap();
        let mstatus = self.csrs.get(&ArchCSRs::mstatus).unwrap();
        let mip = self.csrs.get(&ArchCSRs::mip).unwrap();

        // Check if a timer interrupt is pending
        // timer interrupts are enabled
        // and global interrupts are enabled
        if ((mip.value & MIP_MTIP) != 0)
            && ((mie.value & MIE_MTIE) != 0)
            && ((mstatus.value & MSTATUS_MIE) != 0)
        {
            return Err(RVException::TimerInterrupt);
        }
        Ok(())
    }

    pub fn set_mtip(&mut self, value: bool) {
        const MIP_MTIP: u32 = 1 << 7; // MTIP bit in mip CSR
        let csr = self.csrs.get_mut(&ArchCSRs::mip).unwrap();
        if value {
            csr.value |= MIP_MTIP; // Set the MTIP bit
        } else {
            csr.value &= !MIP_MTIP; // Clear the MTIP bit
        }
    }
}
