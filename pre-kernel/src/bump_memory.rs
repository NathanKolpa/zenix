use core::mem::{align_of, MaybeUninit};

use essentials::address::VirtualAddress;

pub struct BumpMemory {
    start: VirtualAddress,
    end: VirtualAddress,
}

impl BumpMemory {
    pub unsafe fn new(start: VirtualAddress, end: VirtualAddress) -> Self {
        Self { start, end }
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

    pub fn alloc_struct<T>(&mut self) -> &'static mut MaybeUninit<T> {
        let alingment = align_of::<T>();
        let size = align_of::<T>();

        let alignment_offset = alingment % self.start.as_usize();
        let aligned_size = size + alignment_offset;
        let block = self.alloc(aligned_size);

        let aligned_block = &mut block[alignment_offset..aligned_size];

        unsafe { &mut *(aligned_block.as_mut_ptr() as *mut MaybeUninit<T>) }
    }
}
