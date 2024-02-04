use crate::segmentation::*;
use crate::{DescriptorTablePointer, PrivilegeLevel};
use core::mem::size_of;
use essentials::address::VirtualAddress;

#[derive(Debug)]
pub enum SegmentDescriptor {
    Normal(NormalSegment<UserAccessByte>),
    NormalSystem(NormalSegment<SystemAccessByte>),
    LongSystem(LongSegment),
}

impl SegmentDescriptor {
    pub const KERNEL_CODE: Self = SegmentDescriptor::Normal(NormalSegment::KERNEL_CODE);
    pub const KERNEL_DATA: Self = SegmentDescriptor::Normal(NormalSegment::KERNEL_DATA);
    pub const USER_CODE: Self = SegmentDescriptor::Normal(NormalSegment::USER_CODE);
    pub const USER_DATA: Self = SegmentDescriptor::Normal(NormalSegment::USER_DATA);

    pub fn new_tss(tss: &'static TaskStateSegment) -> Self {
        Self::LongSystem(LongSegment::new_tss(tss))
    }

    pub fn privilege(&self) -> PrivilegeLevel {
        match self {
            SegmentDescriptor::Normal(s) => s.privilege(),
            SegmentDescriptor::NormalSystem(s) => s.privilege(),
            SegmentDescriptor::LongSystem(s) => s.privilege(),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            SegmentDescriptor::Normal(_) | SegmentDescriptor::NormalSystem(_) => 1,
            SegmentDescriptor::LongSystem(_) => 2,
        }
    }
}

pub struct GlobalDescriptorTable {
    table: [u64; 10],
    len: usize,
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        Self {
            table: [NormalSegment::NULL.as_u64(); 10],
            len: 1,
        }
    }

    pub fn add_entry(&mut self, descriptor: SegmentDescriptor) -> Option<SegmentSelector> {
        let index = self.len;
        let descriptor_size = descriptor.size();

        if index + descriptor_size > self.table.len() {
            return None;
        }

        self.len += descriptor_size;

        match descriptor {
            SegmentDescriptor::Normal(segment) => {
                self.table[index] = segment.as_u64();
            }
            SegmentDescriptor::NormalSystem(segment) => {
                self.table[index] = segment.as_u64();
            }
            SegmentDescriptor::LongSystem(segment) => {
                let (lower, higher) = segment.as_u64();
                self.table[index] = lower;
                self.table[index + 1] = higher;
            }
        }

        Some(SegmentSelector::new(index as u16, descriptor.privilege()))
    }

    /// Load the table using the `lgdt` instruction.
    pub fn load(&'static self) {
        let pointer = self.pointer();

        unsafe {
            pointer.load_descriptor_table();
        }
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer::new(
            (self.len * size_of::<u64>() - 1) as u16,
            VirtualAddress::from(self as *const _),
        )
    }
}
