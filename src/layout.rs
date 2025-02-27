use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
};

use crate::Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DevId(usize);

pub struct LayoutBuilder {
    max_byte_cnt: usize,
    devs: Vec<Box<dyn Device>>,
    mappings: Vec<MappingRequest>,
}
impl LayoutBuilder {
    pub fn new(max_byte_cnt: usize) -> Self {
        Self {
            max_byte_cnt,
            devs: vec![],
            mappings: vec![],
        }
    }

    pub fn add_device(&mut self, dev: impl Device + 'static) -> DevId {
        let mem_id = DevId(self.devs.len());
        self.devs.push(Box::new(dev));
        mem_id
    }

    pub fn assign(&mut self, addr: usize, mem_id: DevId) -> &mut Self {
        self.assign_range(addr, 1, mem_id)
    }

    pub fn assign_range(&mut self, addr_start: usize, byte_cnt: usize, dev_id: DevId) -> &mut Self {
        if byte_cnt == 0 {
            return self;
        }

        self.mappings.push(MappingRequest {
            addr_start,
            byte_cnt,
            dev_id,
        });

        self
    }

    pub fn build(self) -> Result<Layout, BuildError> {
        // heresy below

        let mut space: Vec<DevId> = vec![DevId(usize::MAX); self.max_byte_cnt];

        for MappingRequest {
            addr_start,
            byte_cnt,
            dev_id,
        } in self.mappings
        {
            if addr_start + byte_cnt > self.max_byte_cnt {
                return Err(BuildError::VirtualAddressOutOfRange(
                    addr_start..(addr_start + byte_cnt),
                ));
            }

            for slot in space.iter_mut().skip(addr_start).take(byte_cnt) {
                *slot = dev_id;
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
                let phys_addr_base = phys_mapping.entry(mem_id).or_default();
                let offset_cnt = i - start;

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
            let phys_addr_base = phys_mapping.entry(mem_id).or_default();

            mappings.insert(
                start,
                Mapping {
                    virtual_addr_start: start,
                    physical_addr_start: *phys_addr_base,
                    mem_id,
                },
            );
        }

        Ok(Layout::new(self.max_byte_cnt, self.devs, mappings))
    }
}

struct MappingRequest {
    addr_start: usize,
    byte_cnt: usize,
    dev_id: DevId,
}

#[derive(Debug)]
pub enum BuildError {
    UnassignedRange(Range<usize>),
    VirtualAddressOutOfRange(Range<usize>),
    MemoryOutOfRange(DevId),
    InvalidMemoryId(DevId),
}

struct Mapping {
    virtual_addr_start: usize,
    physical_addr_start: usize,
    mem_id: DevId,
}

pub struct Layout {
    byte_cnt: usize,
    devs: Vec<Box<dyn Device>>,
    mappings: BTreeMap<usize, Mapping>,
}
impl Layout {
    fn new(
        byte_cnt: usize,
        devs: Vec<Box<dyn Device>>,
        mappings: BTreeMap<usize, Mapping>,
    ) -> Self {
        Self {
            byte_cnt,
            devs,
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
impl Device for Layout {
    fn attach(&mut self) {
        self.devs.iter_mut().for_each(|v| v.attach());
    }

    fn detach(&mut self) {
        self.devs.iter_mut().for_each(|v| v.detach());
    }

    fn reset(&mut self) {
        self.devs.iter_mut().for_each(|v| v.reset());
    }

    fn read(&mut self, addr: usize) -> Option<u8> {
        let Mapping {
            virtual_addr_start,
            physical_addr_start,
            mem_id,
        } = *self.get_mapping_at_addr(addr)?;

        self.devs[mem_id.0].read(physical_addr_start + (addr - virtual_addr_start))
    }

    fn write(&mut self, addr: usize, data: u8) -> Option<()> {
        let Mapping {
            virtual_addr_start,
            physical_addr_start,
            mem_id,
        } = *self.get_mapping_at_addr(addr)?;

        self.devs[mem_id.0].write(physical_addr_start + (addr - virtual_addr_start), data)
    }
}
