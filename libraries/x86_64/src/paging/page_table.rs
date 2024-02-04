use crate::paging::{PageTableEntry, TABLE_ENTRIES};
use core::ops::{Deref, DerefMut};

#[repr(align(4096))]
#[derive(Debug)]
#[repr(C)]
pub struct PageTable {
    entries: [PageTableEntry; TABLE_ENTRIES],
}

impl PageTable {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = PageTableEntry::default()
        }
    }
}

impl Deref for PageTable {
    type Target = [PageTableEntry];

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for PageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}
