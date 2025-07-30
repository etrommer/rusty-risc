use crate::{
    cpu::csr::{self, CSRFile},
    trap::RVException,
};

use super::BusDevice;
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

// https://chromitem-soc.readthedocs.io/en/latest/clint.html

const BASE_ADDR: usize = 0x2000000;
const SIZE: usize = 0xBFFF;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Primitive)]
#[allow(non_camel_case_types)]
pub enum ClintRegisters {
    MTIMECMP_H = 0x4000,
    MTIMECMP_L = 0x4004,
    MTIME_H = 0xBFF8,
    MTIME_L = 0xBFFB,
}

pub struct Clint {
    mtimecmp: u64,
    mtime: u64,
}

impl Clint {
    pub fn new() -> Self {
        Self {
            mtimecmp: u64::MAX,
            mtime: 0,
        }
    }

    pub fn tick(&mut self, csrfile: &mut CSRFile) {
        self.mtime = self.mtime.wrapping_add(1);
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

        match ClintRegisters::from_usize(addr) {
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
        match ClintRegisters::from_usize(addr) {
            Some(ClintRegisters::MTIMECMP_H) => {
                T::to_mem(data, &mut self.mtimecmp.to_le_bytes()[..4]);
            }
            Some(ClintRegisters::MTIMECMP_L) => {
                T::to_mem(data, &mut self.mtimecmp.to_le_bytes()[4..]);
            }
            _ => return Ok(()),
        }
        Ok(())
    }

    fn addr_space(&self) -> (usize, usize) {
        (BASE_ADDR, BASE_ADDR + SIZE)
    }
}
