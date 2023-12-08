use crate::util::address::VirtualAddress;

/// A descriptor table pointer, can either point to a [`super::segmentation::GlobalDescriptorTable`] or [`super::interrupt::InterruptDescriptorTable`].
#[derive(Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    limit: u16,
    base: VirtualAddress,
}

impl DescriptorTablePointer {
    pub const fn new(limit: u16, base: VirtualAddress) -> Self {
        Self { limit, base }
    }

    #[inline]
    pub unsafe fn load_interrupt_table(&self) {
        core::arch::asm!("lidt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }

    #[inline]
    pub unsafe fn load_descriptor_table(&self) {
        core::arch::asm!("lgdt [{}]", in(reg) self, options(readonly, nostack, preserves_flags));
    }
}
