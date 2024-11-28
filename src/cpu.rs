use std::collections::BTreeMap;

use crate::mem::Memory;

#[derive(Debug)]
pub struct TbO2 {
    pc: u16,
    sp: u8,
    a: Register,
    x: Register,
    y: Register,
    status: Status,
    layout: Layout,
}
impl TbO2 {
    pub fn new() -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: Register::default(),
            x: Register::default(),
            y: Register::default(),
            status: Status::default(),
            layout: Layout::new(u16::max_value() as usize + 1),
        }
    }

    pub fn reset(&mut self) {
        self.layout.validate();

        self.status = Status::default();
        self.a = Register::default();
        self.x = Register::default();
        self.y = Register::default();

        self.sp = 0;
        self.pc = 0xFFFC;
    }

    pub fn set_region(&mut self, addr_start: usize, addr_end: usize, mem: Box<dyn Memory>) {
        self.layout.set_region(addr_start, addr_end, mem);
    }
}

#[derive(Debug)]
struct Layout {
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
}
impl Layout {
    pub fn set_region(&mut self, addr_start: usize, addr_end: usize, mem: Box<dyn Memory>) {
        assert!(
            addr_start <= addr_end,
            "addr_end cannot be less than addr_start"
        );
        assert!(
            addr_end - addr_start + 1 <= mem.get_byte_size(),
            "region byte size is too large to fit into the input memory capacity"
        );
        assert!(
            addr_end < self.max_size,
            "addr_end cannot be greater than layout's max byte size"
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

        assert!(
            !self.slots.contains_key(&addr_start),
            "the region is already in used"
        );
        self.slots.insert(addr_start, (addr_end, mem));
    }

    fn validate(&self) {
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

#[derive(Debug, Default)]
struct Status {
    negative: bool,
    overflow: bool,
    break_: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,
}

#[derive(Debug, Default)]
struct Register {
    data: u8,
}
