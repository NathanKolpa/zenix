use crate::memory::alloc::FRAME_ALLOC;
use crate::util::address::PhysicalAddress;

pub struct PhysicalBox {
    address: PhysicalAddress,
}

impl PhysicalBox {
    pub fn new(size: usize) -> Option<(Self, usize)> {
        FRAME_ALLOC
            .allocate(size)
            .map(|(address, size)| (Self { address }, size))
    }

    pub const fn addr(&self) -> PhysicalAddress {
        self.address
    }
}

impl Drop for PhysicalBox {
    fn drop(&mut self) {
        unsafe { FRAME_ALLOC.deallocate(self.address) }
    }
}
