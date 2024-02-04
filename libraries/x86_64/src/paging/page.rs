use core::fmt::Debug;

use crate::paging::PageSize;
use essentials::address::*;

#[derive(Clone, Copy, Debug)]
pub struct Page {
    addr: VirtualAddress,
    size: PageSize,
}

impl Page {
    pub const fn new(addr: VirtualAddress, size: PageSize) -> Self {
        Self {
            addr: addr.align_down(size.as_usize()),
            size,
        }
    }

    pub fn addr(&self) -> VirtualAddress {
        self.addr
    }

    pub fn size(&self) -> PageSize {
        self.size
    }

    pub fn end_addr(&self) -> VirtualAddress {
        self.addr + self.size.as_usize()
    }

    pub fn prev(&self) -> Self {
        Self {
            addr: self.addr - self.size.as_usize(),
            size: self.size,
        }
    }

    pub fn next(&self) -> Self {
        Self {
            addr: self.addr + self.size.as_usize(),
            size: self.size,
        }
    }
}
