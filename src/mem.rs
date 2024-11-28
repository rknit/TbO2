use core::fmt::Debug;

pub trait Memory: Debug {
    fn read_byte(&self, addr: u16) -> u8;

    fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn write_byte(&mut self, addr: u16, data: u8);

    fn write_word(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = ((data & 0xFF00) >> 8) as u8;
        self.write_byte(addr, lo);
        self.write_byte(addr + 1, hi);
    }

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
    fn read_byte(&self, addr: u16) -> u8 {
        let wrapped_addr = (addr as usize) % BYTE_SIZE;
        self.data[wrapped_addr]
    }

    fn write_byte(&mut self, addr: u16, data: u8) {
        let wrapped_addr = (addr as usize) % BYTE_SIZE;
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
    fn read_byte(&self, addr: u16) -> u8 {
        let wrapped_addr = (addr as usize) % BYTE_SIZE;
        self.data[wrapped_addr]
    }

    fn write_byte(&mut self, _addr: u16, _data: u8) {}

    fn get_byte_size(&self) -> usize {
        self.data.len()
    }
}
