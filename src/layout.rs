use std::collections::BTreeMap;

use crate::mem::Memory;

#[derive(Debug)]
pub struct Layout {
    max_size: usize,
    slots: BTreeMap<usize, (usize, Box<dyn Memory>)>,
}
impl Layout {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            slots: BTreeMap::new(),
        }
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        let (offset, mem) = self.get_mem_at_addr(addr);
        mem.read_byte(addr - offset)
    }

    pub fn write_byte(&mut self, addr: usize, data: u8) {
        let (offset, mem) = self.get_mem_at_addr_mut(addr);
        mem.write_byte(addr - offset, data);
    }

    fn get_mem_at_addr(&self, addr: usize) -> (usize, &Box<dyn Memory>) {
        let item = self.slots.range(..=addr).next_back().unwrap();
        (*item.0, &item.1 .1)
    }

    fn get_mem_at_addr_mut(&mut self, addr: usize) -> (usize, &mut Box<dyn Memory>) {
        let item = self.slots.range_mut(..=addr).next_back().unwrap();
        (*item.0, &mut item.1 .1)
    }

    pub fn set_region(&mut self, addr_start: usize, addr_end: usize, mem: Box<dyn Memory>) {
        assert!(
            addr_start <= addr_end,
            "addr_end cannot be less than addr_start"
        );
        assert!(
            addr_end - addr_start + 1 <= mem.get_byte_size(),
            "region byte size is too large to fit into the input memory capacity\
            , addr {:#x} to addr {:#x} requires {} bytes but the memory only has {} bytes",
            addr_start,
            addr_end,
            addr_end - addr_start + 1,
            mem.get_byte_size()
        );
        assert!(
            addr_end < self.max_size,
            "addr_end cannot be greater than layout's max byte size"
        );

        assert!(
            !self.slots.contains_key(&addr_start),
            "the region is already in used"
        );

        let (prev, next) = {
            use std::ops::Bound::*;

            let mut before = self.slots.range((Unbounded, Excluded(addr_start)));
            let mut after = self.slots.range((Excluded(addr_start), Unbounded));

            (before.next_back(), after.next())
        };

        if let Some((_, (end, _))) = prev {
            assert!(*end < addr_start, "region overlapped from the lower addr");
        }
        if let Some((start, _)) = next {
            assert!(*start < addr_end, "region overlapped from the higher addr");
        }

        self.slots.insert(addr_start, (addr_end, mem));
    }

    pub fn validate(&self) {
        let mut prev_end: Option<usize> = None;
        for (start, (end, _)) in &self.slots {
            if let Some(prev_end) = prev_end {
                assert!(
                    start - prev_end == 1,
                    "undefined memory region from addr {:#x} to {:#x}",
                    prev_end + 1,
                    start - 1
                );
            } else {
                assert!(
                    *start == 0,
                    "undefined memory region from addr {:#x} to {:#x}",
                    0,
                    start - 1,
                )
            }
            prev_end = Some(*end);
        }
        if let Some(prev_end) = prev_end {
            assert!(
                prev_end == self.max_size - 1,
                "undefined memory region from addr {:#x} to {:#x}",
                prev_end + 1,
                self.max_size - 1
            );
        } else {
            panic!(
                "undefined memory region from addr {:#x} to {:#x}",
                0,
                self.max_size - 1
            );
        }
    }
}
