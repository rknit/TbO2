use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
};

use crate::mem::Memory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemId(usize);

pub struct LayoutBuilder {
    max_byte_cnt: usize,
    mems: Vec<Box<dyn Memory>>,
    mappings: Vec<MappingRequest>,
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

        self.mappings.push(MappingRequest {
            addr_start,
            byte_cnt,
            mem_id,
        });

        self
    }

    pub fn build(self) -> Result<Layout, BuildError> {
        // heresy below

        let mut space: Vec<MemId> = vec![MemId(usize::MAX); self.max_byte_cnt];

        for MappingRequest {
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

            if byte_cnt > mem.get_byte_count() {
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
        let mut phys_mapping = HashMap::new();
        let mut start = 0;
        let mut mem_id = space[0];

        for (i, slot) in space.into_iter().enumerate() {
            if slot != mem_id {
                let mem = self.mems.get(mem_id.0).unwrap();
                let phys_addr_base = phys_mapping.entry(mem_id).or_default();
                let offset_cnt = i - start;

                if *phys_addr_base + offset_cnt > mem.get_byte_count() {
                    return Err(BuildError::MemoryOutOfRange(mem_id));
                }

                mappings.insert(
                    start,
                    Mapping {
                        virtual_addr_start: start,
                        physical_addr_start: *phys_addr_base,
                        mem_id,
                    },
                );
                *phys_addr_base += offset_cnt;
                mem_id = slot;
                start = i;
            }
        }

        {
            let mem = self.mems.get(mem_id.0).unwrap();
            let phys_addr_base = phys_mapping.entry(mem_id).or_default();
            let offset_cnt = self.max_byte_cnt - start;

            if *phys_addr_base + offset_cnt > mem.get_byte_count() {
                return Err(BuildError::MemoryOutOfRange(mem_id));
            }

            mappings.insert(
                start,
                Mapping {
                    virtual_addr_start: start,
                    physical_addr_start: *phys_addr_base,
                    mem_id,
                },
            );
        }

        Ok(Layout::new(self.max_byte_cnt, self.mems, mappings))
    }
}

struct MappingRequest {
    addr_start: usize,
    byte_cnt: usize,
    mem_id: MemId,
}

#[derive(Debug)]
pub enum BuildError {
    UnassignedRange(Range<usize>),
    VirtualAddressOutOfRange(Range<usize>),
    MemoryOutOfRange(MemId),
    InvalidMemoryId(MemId),
}

struct Mapping {
    virtual_addr_start: usize,
    physical_addr_start: usize,
    mem_id: MemId,
}

pub struct Layout {
    byte_cnt: usize,
    mems: Vec<Box<dyn Memory>>,
    mappings: BTreeMap<usize, Mapping>,
}
impl Layout {
    fn new(
        byte_cnt: usize,
        mems: Vec<Box<dyn Memory>>,
        mappings: BTreeMap<usize, Mapping>,
    ) -> Self {
        Self {
            byte_cnt,
            mems,
            mappings,
        }
    }

    pub fn get_byte_count(&self) -> usize {
        self.byte_cnt
    }

    fn get_mapping_at_addr(&self, addr: usize) -> Option<&Mapping> {
        self.mappings.range(..=addr).next_back().map(|v| v.1)
    }
}
impl Memory for Layout {
    fn read_byte(&self, addr: usize) -> Option<u8> {
        let Mapping {
            virtual_addr_start,
            physical_addr_start,
            mem_id,
        } = self.get_mapping_at_addr(addr)?;

        self.mems[mem_id.0].read_byte(physical_addr_start + (addr - virtual_addr_start))
    }

    fn write_byte(&mut self, addr: usize, data: u8) -> Option<()> {
        let Mapping {
            virtual_addr_start,
            physical_addr_start,
            mem_id,
        } = *self.get_mapping_at_addr(addr)?;

        self.mems[mem_id.0].write_byte(physical_addr_start + (addr - virtual_addr_start), data)
    }

    fn get_byte_count(&self) -> usize {
        self.byte_cnt
    }
}
