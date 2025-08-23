use super::BusDevice;
use super::BusError;

pub struct Ram {
    addr_space: (usize, usize),
    pub mem: Vec<u8>,
}

impl Ram {
    pub fn new(ram: Vec<u8>, ram_start: usize) -> Self {
        Self {
            addr_space: (ram_start, ram_start + ram.len()),
            mem: ram,
        }
    }
}

impl BusDevice for Ram {
    fn load<T: super::BusWidth<T> + std::fmt::Display>(&self, addr: usize) -> Result<T, BusError> {
        let uaddr = addr as usize - self.addr_space.0;
        let value = T::from_mem(&self.mem[uaddr..uaddr + T::WIDTH]);
        Ok(value)
    }

    fn store<T: super::BusWidth<T> + std::fmt::Display>(
        &mut self,
        addr: usize,
        data: T,
    ) -> Result<(), BusError> {
        let uaddr = addr as usize - self.addr_space.0;
        T::to_mem(data, &mut self.mem[uaddr..uaddr + T::WIDTH]);
        Ok(())
    }

    fn addr_space(&self) -> (usize, usize) {
        self.addr_space
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_write() {
        let mut dut: Ram = Ram {
            addr_space: (0, 0),
            mem: vec![0; 4],
        };

        assert_eq!(dut.store::<u32>(0, 0xaabbccdd), Ok(()));
        assert_eq!(dut.mem[0..4], vec![0xdd, 0xcc, 0xbb, 0xaa]);

        assert_eq!(dut.store::<u16>(0, 0x0011), Ok(()));
        assert_eq!(dut.mem[0..4], vec![0x11, 0x00, 0xbb, 0xaa]);

        assert_eq!(dut.store::<u8>(0, 0xee), Ok(()));
        assert_eq!(dut.mem[0..4], vec![0xee, 0x00, 0xbb, 0xaa]);
    }
    #[test]
    fn test_read() {
        let mut dut: Ram = Ram {
            addr_space: (0, 0),
            mem: vec![0; 4],
        };

        dut.mem = vec![0xaa, 0xbb, 0xcc, 0xdd];
        assert_eq!(dut.load::<u32>(0), Ok(0xddccbbaa));
        assert_eq!(dut.load::<u16>(0), Ok(0xbbaa));
        assert_eq!(dut.load::<u16>(2), Ok(0xddcc));
        assert_eq!(dut.load::<u8>(0), Ok(0xaa));
        assert_eq!(dut.load::<u8>(2), Ok(0xcc));
    }
}
