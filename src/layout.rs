use std::{collections::BTreeMap, ops::Range};

use crate::mem::Memory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemId(usize);

pub struct LayoutBuilder {
    max_byte_cnt: usize,
    mems: Vec<Box<dyn Memory>>,
    mappings: Vec<Mapping>,
}
impl LayoutBuilder {
    pub fn new(max_byte_cnt: usize) -> Self {
        Self {
            max_byte_cnt,
            mems: vec![],
            mappings: vec![],
        }
    }

    pub fn add_memory(&mut self, mem: impl Memory + 'static) -> MemId {
        let mem_id = MemId(self.mems.len());
        self.mems.push(Box::new(mem));
        mem_id
    }

    pub fn assign(&mut self, addr: usize, mem_id: MemId) -> &mut Self {
        self.assign_range(addr, 1, mem_id)
    }

    pub fn assign_range(&mut self, addr_start: usize, byte_cnt: usize, mem_id: MemId) -> &mut Self {
        if byte_cnt == 0 {
            return self;
        }

        self.mappings.push(Mapping {
            addr_start,
            byte_cnt,
            mem_id,
        });

        self
    }

    pub fn build(self) -> Result<Layout, BuildError> {
        // heresy below

        let mut space: Vec<MemId> = vec![MemId(usize::MAX); self.max_byte_cnt];

        for Mapping {
            addr_start,
            byte_cnt,
            mem_id,
        } in self.mappings
        {
            if addr_start + byte_cnt > self.max_byte_cnt {
                return Err(BuildError::VirtualAddressOutOfRange(
                    addr_start..(addr_start + byte_cnt),
                ));
            }

            let mem = self
                .mems
                .get(mem_id.0)
                .ok_or(BuildError::InvalidMemoryId(mem_id))?;

            if addr_start + byte_cnt > mem.get_byte_count() {
                return Err(BuildError::MemoryOutOfRange(mem_id));
            }

            for slot in space.iter_mut().skip(addr_start).take(byte_cnt) {
                *slot = mem_id;
            }
        }

        for (i, slot) in space.iter().enumerate() {
            if slot.0 == usize::MAX {
                let range = space.iter().skip(i + 1).take_while(|v| v.0 == usize::MAX);
                return Err(BuildError::UnassignedRange(i..(i + range.count())));
            }
        }

        let mut mappings = BTreeMap::new();
        let mut start = 0;
        let mut mem_id = space[0];

        for (i, slot) in space.into_iter().enumerate() {
            if slot != mem_id {
                mappings.insert(start, mem_id);
                mem_id = slot;
                start = i;
            }
        }
        mappings.insert(start, mem_id);

        Ok(Layout::new(self.max_byte_cnt, self.mems, mappings))
    }
}

#[derive(Debug, Clone)]
struct Mapping {
    addr_start: usize,
    byte_cnt: usize,
    mem_id: MemId,
}

pub enum BuildError {
    UnassignedRange(Range<usize>),
    VirtualAddressOutOfRange(Range<usize>),
    MemoryOutOfRange(MemId),
    InvalidMemoryId(MemId),
}

pub struct Layout {
    byte_cnt: usize,
    mems: Vec<Box<dyn Memory>>,
    mappings: BTreeMap<usize, MemId>,
}
impl Layout {
    fn new(byte_cnt: usize, mems: Vec<Box<dyn Memory>>, mappings: BTreeMap<usize, MemId>) -> Self {
        Self {
            byte_cnt,
            mems,
            mappings,
        }
    }

    pub fn get_byte_count(&self) -> usize {
        self.byte_cnt
    }

    fn get_mem_id_at_addr(&self, addr: usize) -> Option<&MemId> {
        self.mappings.range(..=addr).next_back().map(|v| v.1)
    }

    fn get_mem_at_addr(&self, addr: usize) -> Option<&dyn Memory> {
        self.mems
            .get(self.get_mem_id_at_addr(addr)?.0)
            .map(move |v| v.as_ref())
    }

    fn get_mem_at_addr_mut(&mut self, addr: usize) -> Option<&mut dyn Memory> {
        let idx = self.get_mem_id_at_addr(addr)?.0;
        self.mems
            .get_mut(idx)
            .map(move |v| -> &mut dyn Memory { v.as_mut() })
    }
}
impl Memory for Layout {
    fn read_byte(&self, addr: usize) -> Option<u8> {
        self.get_mem_at_addr(addr)?.read_byte(addr)
    }

    fn write_byte(&mut self, addr: usize, data: u8) -> Option<()> {
        self.get_mem_at_addr_mut(addr)?.write_byte(addr, data)
    }

    fn get_byte_count(&self) -> usize {
        self.byte_cnt
    }
}
