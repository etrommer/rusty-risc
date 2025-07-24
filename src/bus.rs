pub mod clint;
pub mod ram;
pub mod uart;

use self::ram::Ram;
use self::uart::Uart;

use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum BusError {
    AddressMisaligned(usize),
    AddressUnmapped(usize),
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (optype, addr) = match self {
            Self::AddressMisaligned(addr) => ("Misaligned", addr),
            Self::AddressUnmapped(addr) => ("Unmapped", addr),
        };
        write! {f, "BusError {} @ {:>08x}", optype, addr}
    }
}

use std::convert::TryFrom;

pub trait BusWidth<T> {
    const WIDTH: usize;
    fn is_aligned(addr: usize) -> bool;
    fn to_mem(val: T, dst: &mut [u8]);
    fn from_mem(bytes: &[u8]) -> T;
}

macro_rules! buswidth {
    ($t : ty) => {
        impl BusWidth<$t> for $t {
            const WIDTH: usize = std::mem::size_of::<$t>();

            fn is_aligned(addr: usize) -> bool {
                (addr as usize % Self::WIDTH) == 0
            }

            fn to_mem(val: $t, dst: &mut [u8]) {
                for (d, b) in std::iter::zip(dst.iter_mut(), val.to_le_bytes()) {
                    *d = b
                }
            }

            fn from_mem(bytes: &[u8]) -> $t {
                let array = <[u8; Self::WIDTH]>::try_from(bytes).unwrap_or([0; Self::WIDTH]);
                <$t>::from_le_bytes(array)
            }
        }
    };
}

buswidth!(u32);
buswidth!(i32);
buswidth!(u16);
buswidth!(i16);
buswidth!(u8);
buswidth!(i8);

pub trait BusDevice {
    fn addr_space(&self) -> (usize, usize);
    fn load<T: BusWidth<T> + std::fmt::Display>(&self, addr: usize) -> Result<T, BusError>;
    fn store<T: BusWidth<T> + std::fmt::Display>(
        &mut self,
        addr: usize,
        data: T,
    ) -> Result<(), BusError>;
}

pub struct Bus {
    ram: Ram,
    uart: Uart,
}

impl Bus {
    pub fn new(ram: Vec<u8>) -> Self {
        Self {
            ram: Ram::new(ram),
            uart: Uart {},
        }
    }
}

impl BusDevice for Bus {
    fn load<T: BusWidth<T> + std::fmt::Display>(&self, addr: usize) -> Result<T, BusError> {
        if !T::is_aligned(addr) {
            return Err(BusError::AddressMisaligned(addr));
        }
        // TODO: Iterate Bus Devices
        let (ram_lower, ram_upper) = self.ram.addr_space();
        if addr >= ram_lower && addr < ram_upper {
            return self.ram.load(addr);
        }
        let (uart_lower, uart_upper) = self.uart.addr_space();
        if addr >= uart_lower && addr < uart_upper {
            return self.uart.load(addr);
        }

        // Load from unmapped address
        Err(BusError::AddressUnmapped(addr))
    }

    fn store<T: BusWidth<T> + std::fmt::Display>(
        &mut self,
        addr: usize,
        data: T,
    ) -> Result<(), BusError> {
        if !T::is_aligned(addr) {
            return Err(BusError::AddressMisaligned(addr));
        }
        let (ram_lower, ram_upper) = self.ram.addr_space();
        // TODO: Iterate Bus Devices
        if addr >= ram_lower && addr < ram_upper {
            return self.ram.store(addr, data);
        }
        let (uart_lower, uart_upper) = self.uart.addr_space();
        if addr >= uart_lower && addr < uart_upper {
            return self.uart.store(addr, data);
        }

        // Store to unmapped address
        Err(BusError::AddressUnmapped(addr))
    }

    fn addr_space(&self) -> (usize, usize) {
        return (0, usize::MAX);
    }
}
