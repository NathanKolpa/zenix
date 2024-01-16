use crate::arch::x86_64::paging::{PageTableEntry, TABLE_ENTRIES};
use core::ops::{Deref, DerefMut};

const NORMAL_TABLE_ENTRIES: usize = TABLE_ENTRIES;
const HUGE_L1_TABLE_ENTRIES: usize = TABLE_ENTRIES * TABLE_ENTRIES;
const HUGE_L2_TABLE_ENTRIES: usize = HUGE_L1_TABLE_ENTRIES * TABLE_ENTRIES;

#[repr(align(4096))]
#[derive(Debug)]
#[repr(C)]
pub struct PageTable<const SIZE: usize = NORMAL_TABLE_ENTRIES> {
    entries: [PageTableEntry; SIZE],
}

impl<const SIZE: usize> PageTable<SIZE> {
    pub fn as_slice(&self) -> &[PageTableEntry] {
        &self.entries
    }

    pub fn as_mut_slice(&mut self) -> &mut [PageTableEntry] {
        &mut self.entries
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = PageTableEntry::default()
        }
    }
}

impl<const SIZE: usize> Deref for PageTable<SIZE> {
    type Target = [PageTableEntry];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const SIZE: usize> DerefMut for PageTable<SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

pub type HugeL1Table = PageTable<HUGE_L1_TABLE_ENTRIES>;
pub type HugeL2Table = PageTable<HUGE_L2_TABLE_ENTRIES>;
