#[derive(Debug)]
pub struct RegFile {
    registers: [i32; 31],
}

impl RegFile {
    pub fn new() -> Self {
        Self { registers: [0; 31] }
    }

    pub fn read(&self, num: usize) -> i32 {
        if num == 0 {
            0
        } else {
            self.registers[num - 1]
        }
    }

    pub fn write(&mut self, num: usize, value: i32) {
        if num > 0 {
            self.registers[num - 1] = value;
        }
    }

    // pub fn write(&mut self, num: u32, value: i32) {
    //     self.write(num, value as i32)
    // }
}
