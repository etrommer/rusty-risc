use super::BusDevice;
use super::BusError;
use std::convert::TryInto;

// https://chromitem-soc.readthedocs.io/en/latest/clint.html

const BASE_ADDR: usize = 0x10000000;

pub enum ClintRegisters {
    MSIP = 0x0,
    MTIMECMP = 0x4000,
    MTIME = 0xBFF8,
}

pub struct Clint {
    msip: u32,
    mtimecmp: u64,
    mtime: u64,
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
        Ok(T::from_mem(&[0]))
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
        Ok(())
    }

    fn addr_space(&self) -> (usize, usize) {
        (BASE_ADDR, BASE_ADDR + 0xbfff)
    }
}
