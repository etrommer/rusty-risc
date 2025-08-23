use crate::cpu::csr::CSRFile;
use std::time::Instant;

use super::BusDevice;
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

// https://chromitem-soc.readthedocs.io/en/latest/clint.html
const BASE_ADDR: usize = 0x1100_0000;
const SIZE: usize = 0xBFFF;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Primitive)]
#[allow(non_camel_case_types)]
pub enum ClintRegisters {
    MTIMECMP_H = 0x4004,
    MTIMECMP_L = 0x4000,
    MTIME_H = 0xBFFC,
    MTIME_L = 0xBFF8,
}

pub struct Clint {
    start_time: Instant,
    mtimecmp: u64,
    mtime: u64,
}

impl Clint {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            mtimecmp: u32::MAX as u64,
            mtime: 0,
        }
    }

    pub fn tick(&mut self, csrfile: &mut CSRFile) {
        self.mtime = self.start_time.elapsed().as_micros() as u64;
        if self.mtime >= self.mtimecmp {
            csrfile.set_mtip(true);
        } else {
            csrfile.set_mtip(false);
        }
    }
}

impl BusDevice for Clint {
    fn load<T: super::BusWidth<T> + std::fmt::Display>(
        &self,
        addr: usize,
    ) -> Result<T, super::BusError> {
        let offset = addr - BASE_ADDR;
        if !T::is_aligned(offset) {
            return Err(super::BusError::AddressMisaligned(addr));
        }

        match ClintRegisters::from_usize(offset) {
            Some(ClintRegisters::MTIMECMP_H) => {
                return Ok(T::from_mem(&self.mtimecmp.to_le_bytes()[4..]));
            }
            Some(ClintRegisters::MTIMECMP_L) => {
                return Ok(T::from_mem(&self.mtimecmp.to_le_bytes()[..4]));
            }
            Some(ClintRegisters::MTIME_H) => {
                return Ok(T::from_mem(&self.mtime.to_le_bytes()[4..]));
            }
            Some(ClintRegisters::MTIME_L) => {
                return Ok(T::from_mem(&self.mtime.to_le_bytes()[..4]));
            }
            None => return Ok(T::from_mem(&[0])),
        }
    }

    fn store<T: super::BusWidth<T> + std::fmt::Display>(
        &mut self,
        addr: usize,
        data: T,
    ) -> Result<(), super::BusError> {
        let offset = addr - BASE_ADDR;
        if !T::is_aligned(offset) {
            return Err(super::BusError::AddressMisaligned(addr));
        }
        match ClintRegisters::from_usize(offset) {
            Some(ClintRegisters::MTIMECMP_H) => {
                let mut new_high = [0u8; 4];
                T::to_mem(data, &mut new_high);
                self.mtimecmp = (u32::from_le_bytes(new_high) as u64) << 32
                    | (self.mtimecmp & 0x0000_0000_FFFF_FFFF);
            }
            Some(ClintRegisters::MTIMECMP_L) => {
                let mut new_low = [0u8; 4];
                T::to_mem(data, &mut new_low);
                self.mtimecmp =
                    (u32::from_le_bytes(new_low) as u64) | (self.mtimecmp & 0xFFFF_FFFF_0000_0000);
            }
            _ => (),
        }
        Ok(())
    }

    fn addr_space(&self) -> (usize, usize) {
        (BASE_ADDR, BASE_ADDR + SIZE)
    }
}
