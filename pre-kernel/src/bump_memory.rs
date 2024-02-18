use core::{
    mem::{align_of, size_of, MaybeUninit},
    u8,
};

use bootinfo::MemoryRegion;
use essentials::address::VirtualAddress;

use crate::regions::{BUMP_MEMORY_END, BUMP_MEMORY_START};

pub struct BumpMemory {
    initial_start: VirtualAddress,
    start: VirtualAddress,
    end: VirtualAddress,
}

impl BumpMemory {
    pub unsafe fn new(start: VirtualAddress, end: VirtualAddress) -> Self {
        Self {
            initial_start: start,
            start,
            end,
        }
    }

    pub unsafe fn new_from_linker() -> Self {
        Self::new(
            VirtualAddress::from(&BUMP_MEMORY_START as *const _),
            VirtualAddress::from(&BUMP_MEMORY_END as *const _),
        )
    }

    fn move_start_to_new_pos(&mut self, size: usize) -> VirtualAddress {
        if self.start + size >= self.end {
            panic!("Out of bump memory");
        }

        let current_start = self.start;
        self.start += size;
        current_start
    }

    pub fn alloc(&mut self, size: usize) -> &'static mut [u8] {
        let start = self.move_start_to_new_pos(size);
        unsafe { core::slice::from_raw_parts_mut(start.as_mut_ptr(), size) }
    }

    pub fn alloc_aligned(&mut self, size: usize, alignment: usize) -> &'static mut [u8] {
        let alignment_offset = self.start.as_usize() % alignment;

        if alignment_offset > 0 {
            self.start += alignment - alignment_offset;
        }

        self.alloc(size)
    }

    pub fn alloc_struct<T>(&mut self) -> &'static mut MaybeUninit<T> {
        let alingment = align_of::<T>();
        let size = size_of::<T>();

        let block = self.alloc_aligned(size, alingment);

        debug_assert_eq!(block.len(), size);
        debug_assert_eq!(block.as_ptr() as usize % alingment, 0);

        unsafe { &mut *(block.as_mut_ptr() as *mut MaybeUninit<T>) }
    }

    pub fn left_over_memory(&self) -> MemoryRegion {
        MemoryRegion {
            start: self.start.as_u64(),
            size: (self.end - self.start).as_u64(),
        }
    }

    pub fn used_memory(&self) -> MemoryRegion {
        MemoryRegion {
            start: self.initial_start.as_u64(),
            size: (self.start - self.initial_start).as_u64(),
        }
    }
}
