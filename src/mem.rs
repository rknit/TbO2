use core::fmt::Debug;

pub trait Memory: Debug {
    fn read_byte(&self, addr: usize) -> u8;

    fn write_byte(&mut self, addr: usize, data: u8);

    fn get_byte_size(&self) -> usize;
}

#[derive(Debug)]
pub struct RAM<const BYTE_SIZE: usize> {
    data: [u8; BYTE_SIZE],
}
impl<const BYTE_SIZE: usize> RAM<BYTE_SIZE> {
    pub fn new() -> Self {
        Self {
            data: [0; BYTE_SIZE],
        }
    }
}
impl<const BYTE_SIZE: usize> Memory for RAM<BYTE_SIZE> {
    fn read_byte(&self, addr: usize) -> u8 {
        let wrapped_addr = addr % BYTE_SIZE;
        self.data[wrapped_addr]
    }

    fn write_byte(&mut self, addr: usize, data: u8) {
        let wrapped_addr = addr % BYTE_SIZE;
        self.data[wrapped_addr] = data;
    }

    fn get_byte_size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug)]
pub struct ROM<const BYTE_SIZE: usize> {
    data: [u8; BYTE_SIZE],
}
impl<const BYTE_SIZE: usize> ROM<BYTE_SIZE> {
    pub fn new() -> Self {
        Self {
            data: [0; BYTE_SIZE],
        }
    }
}
impl<const BYTE_SIZE: usize> Memory for ROM<BYTE_SIZE> {
    fn read_byte(&self, addr: usize) -> u8 {
        let wrapped_addr = addr % BYTE_SIZE;
        self.data[wrapped_addr]
    }

    fn write_byte(&mut self, _addr: usize, _data: u8) {}

    fn get_byte_size(&self) -> usize {
        self.data.len()
    }
}
