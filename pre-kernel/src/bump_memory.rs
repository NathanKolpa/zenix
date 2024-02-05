use core::mem::{align_of, MaybeUninit};

use essentials::address::VirtualAddress;

use crate::vga::{VGA_ADDR, VGA_LEN};

// has to be sorted
const UNUSABLE_REGIONS: &[(VirtualAddress, usize)] = &[(VGA_ADDR, VGA_LEN)];

pub struct BumpMemory {
    start: VirtualAddress,
    end: VirtualAddress,
    unusable_regions: &'static [(VirtualAddress, usize)],
}

impl BumpMemory {
    pub unsafe fn new(start: VirtualAddress, end: VirtualAddress) -> Self {
        Self {
            start,
            end,
            unusable_regions: UNUSABLE_REGIONS,
        }
    }

    fn move_start_to_new_pos(&mut self, size: usize) -> VirtualAddress {
        loop {
            let new_end = self.start + size;

            if let Some((region_addr, region_len)) = self.unusable_regions.first() {
                if new_end.is_within(*region_addr, *region_len) {
                    self.start = *region_addr + *region_len;
                    self.unusable_regions = &self.unusable_regions[1..self.unusable_regions.len()];
                    continue;
                }
            }

            break;
        }

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
