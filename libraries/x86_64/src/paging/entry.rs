use core::{
    fmt::{Debug, Formatter},
    u64,
};

use crate::paging::PageTableEntryFlags;
use essentials::address::PhysicalAddress;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PageTableEntry {
    pub value: u64,
}

impl PageTableEntry {
    const ADDR_MASK: u64 = 0x000f_ffff_ffff_f000;
    const FLAGS_MASK: u64 = !Self::ADDR_MASK;

    pub const fn new_u64_addr(flags: PageTableEntryFlags, addr: u64) -> Self {
        let flags_masked = flags.as_u64() & Self::FLAGS_MASK;
        let addr_masked = addr & Self::ADDR_MASK;

        Self {
            value: flags_masked | addr_masked,
        }
    }

    pub const fn new(flags: PageTableEntryFlags, addr: PhysicalAddress) -> Self {
        Self::new_u64_addr(flags, addr.as_u64())
    }

    pub fn set_flags(&mut self, flags: PageTableEntryFlags) {
        self.value = self.value ^ ((self.value ^ flags.as_u64()) & Self::FLAGS_MASK);
    }

    pub fn set_addr_u64(&mut self, addr: u64) {
        self.value = self.value ^ ((self.value ^ addr) & Self::ADDR_MASK);
    }

    #[cfg(target_arch = "x86_64")]
    #[doc(cfg(target_arch = "x86_64"))]
    pub fn set_addr(&mut self, addr: PhysicalAddress) {
        self.set_addr_u64(addr.as_u64())
    }

    pub const fn flags(&self) -> PageTableEntryFlags {
        PageTableEntryFlags::new(self.value & Self::FLAGS_MASK)
    }

    pub const fn addr_u64(&self) -> u64 {
        self.value & Self::ADDR_MASK
    }

    #[cfg(target_arch = "x86_64")]
    #[doc(cfg(target_arch = "x86_64"))]
    pub const fn addr(&self) -> PhysicalAddress {
        PhysicalAddress::new(self.addr_u64() as usize)
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("addr", &self.addr_u64())
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
