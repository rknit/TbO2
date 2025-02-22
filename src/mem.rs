pub trait Memory {
    #[must_use]
    fn read_byte(&self, addr: usize) -> Option<u8>;

    fn write_byte(&mut self, addr: usize, data: u8) -> Option<()>;

    fn get_byte_count(&self) -> usize;
}

pub struct RAM<const BYTE_CNT: usize> {
    data: [u8; BYTE_CNT],
}
impl<const BYTE_CNT: usize> Default for RAM<BYTE_CNT> {
    fn default() -> Self {
        Self {
            data: [0; BYTE_CNT],
        }
    }
}
impl<const BYTE_CNT: usize> RAM<BYTE_CNT> {
    pub fn load_bytes(&mut self, addr_start: usize, data: &[u8]) {
        assert!(
            addr_start + data.len() <= BYTE_CNT,
            "ending address ({:#0x}) exceeds the capacity ({})",
            addr_start + data.len(),
            BYTE_CNT
        );
        self.data
            .iter_mut()
            .skip(addr_start)
            .zip(data)
            .for_each(|(to, from)| *to = *from);
    }
}
impl<const BYTE_CNT: usize> Memory for RAM<BYTE_CNT> {
    fn read_byte(&self, addr: usize) -> Option<u8> {
        let wrapped_addr = addr % BYTE_CNT;
        Some(self.data[wrapped_addr])
    }

    fn write_byte(&mut self, addr: usize, data: u8) -> Option<()> {
        let wrapped_addr = addr % BYTE_CNT;
        self.data[wrapped_addr] = data;
        Some(())
    }

    fn get_byte_count(&self) -> usize {
        self.data.len()
    }
}

pub struct ROM<const BYTE_CNT: usize> {
    data: [u8; BYTE_CNT],
}
impl<const BYTE_CNT: usize> Default for ROM<BYTE_CNT> {
    fn default() -> Self {
        Self {
            data: [0; BYTE_CNT],
        }
    }
}
impl<const BYTE_CNT: usize> ROM<BYTE_CNT> {
    pub fn load_bytes(&mut self, addr_start: usize, data: &[u8]) {
        assert!(
            addr_start + data.len() <= BYTE_CNT,
            "ending address ({:#0x}) exceeds the capacity ({})",
            addr_start + data.len(),
            BYTE_CNT
        );
        self.data
            .iter_mut()
            .skip(addr_start)
            .zip(data)
            .for_each(|(to, from)| *to = *from);
    }
}
impl<const BYTE_CNT: usize> Memory for ROM<BYTE_CNT> {
    fn read_byte(&self, addr: usize) -> Option<u8> {
        let wrapped_addr = addr % BYTE_CNT;
        Some(self.data[wrapped_addr])
    }

    fn write_byte(&mut self, _addr: usize, _data: u8) -> Option<()> {
        None
    }

    fn get_byte_count(&self) -> usize {
        self.data.len()
    }
}
