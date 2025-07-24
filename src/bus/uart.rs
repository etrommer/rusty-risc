use super::BusDevice;

const BASE_ADDR: usize = 0x10000000;

pub struct Uart {}

impl BusDevice for Uart {
    fn load<T: super::BusWidth<T> + std::fmt::Display>(
        &self,
        addr: usize,
    ) -> Result<T, super::BusError> {
        if addr == BASE_ADDR + 0x5 {
            return Ok(T::from_mem(&[0x40]));
        }
        Ok(T::from_mem(&[0x00]))
    }

    fn store<T: super::BusWidth<T> + std::fmt::Display>(
        &mut self,
        addr: usize,
        data: T,
    ) -> Result<(), super::BusError> {
        if addr == BASE_ADDR && T::WIDTH == 1 {
            // Not very elegant way of extracting u8 from generic type `T`
            let mut c = [0u8];
            T::to_mem(data, &mut c);
            print!("{}", c[0] as char);
        }
        Ok(())
    }

    fn addr_space(&self) -> (usize, usize) {
        (BASE_ADDR, BASE_ADDR + 0xff)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_write() {
        let mut dut: Uart = Uart {};

        assert_eq!(dut.store::<u8>(BASE_ADDR, b'a'), Ok(()));
    }
    #[test]
    fn test_read() {
        let dut: Uart = Uart {};

        assert_eq!(dut.load::<u8>(BASE_ADDR + 0x05), Ok(0x40));
    }
}
