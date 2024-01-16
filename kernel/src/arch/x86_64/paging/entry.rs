use core::fmt::{Debug, Display, Formatter};

use crate::arch::x86_64::paging::PageTableEntryFlags;
use crate::util::address::PhysicalAddress;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;
    const FLAGS_MASK: u64 = !Self::ADDR_MASK;

    pub const fn new(flags: PageTableEntryFlags, addr: PhysicalAddress) -> Self {
        let flags_masked = flags.as_u64() & Self::FLAGS_MASK;
        let addr_masked = addr.as_u64() & Self::ADDR_MASK;

        Self {
            value: flags_masked | addr_masked,
        }
    }

    pub fn set_flags(&mut self, flags: PageTableEntryFlags) {
        self.value = self.value ^ ((self.value ^ flags.as_u64()) & Self::FLAGS_MASK);
    }

    pub fn set_addr(&mut self, addr: PhysicalAddress) {
        self.value = self.value ^ ((self.value ^ addr.as_u64()) & Self::ADDR_MASK);
    }

    pub const fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags::new(self.value & Self::FLAGS_MASK)
    }

    pub const fn addr(&self) -> PhysicalAddress {
        PhysicalAddress::new((self.value & Self::ADDR_MASK) as usize)
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("addr", &self.addr())
            .field("flags", &self.flags())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_entry_new_and_getters() {
        let flags = PageTableEntryFlags::null()
            .set_writable(true)
            .set_no_exec(true)
            .set_custom::<10>(true);

        let addr = PhysicalAddress::new(4096);

        let entry = PageTableEntry::new(flags, addr);

        assert_eq!(flags, entry.flags());
        assert_eq!(addr, entry.addr());
    }
}
