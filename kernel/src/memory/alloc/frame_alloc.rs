use crate::memory::alloc::MemoryInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use crate::util::{FixedVec, PanicOnce};

pub struct AllocatedFrame {
    alloc: &'static FrameAllocator<'static>,
    size: usize,
}

impl Drop for AllocatedFrame {
    fn drop(&mut self) {
        self.alloc.deallocate(self.size);
    }
}

struct FreeList {}

impl FreeList {
    pub const fn new() -> Self {
        Self {}
    }
}

/// This is where **all** the memory comes from :^)
///
/// A frame allocator
/// using the [Buddy Allocator Algorithm](https://en.wikipedia.org/wiki/Buddy_memory_allocation),
/// this is the same algorithm that the Linux kernel uses.
pub struct FrameAllocator<'a> {
    memory_map: PanicOnce<&'a MemoryMap>,
    slots: FixedVec<30, FreeList>,
}

impl<'a> FrameAllocator<'a> {
    const fn new() -> Self {
        Self {
            memory_map: PanicOnce::new(),
            slots: FixedVec::new(),
        }
    }

    pub fn init_with(&self, map: &'a MemoryMap) {
        self.memory_map.initialize_with(map);
    }

    pub fn info(&self) -> MemoryInfo {
        let bytes_allocated = 0;

        let mut total_allocatable_bytes = 0;
        let mut total_bytes = 0;
        let mut kernel = 0;

        let regions = self.memory_map.iter().map(|x| {
            (
                x.region_type,
                (x.range.end_frame_number as usize * 4096
                    - x.range.start_frame_number as usize * 4096),
            )
        });

        for (kind, region_size) in regions {
            match &kind {
                MemoryRegionType::Usable => total_allocatable_bytes += region_size,
                MemoryRegionType::KernelStack | MemoryRegionType::Kernel => kernel += region_size,
                _ => {}
            }

            if kind != MemoryRegionType::Reserved {
                total_bytes += region_size;
            }
        }

        MemoryInfo {
            allocated: bytes_allocated,
            usable: total_allocatable_bytes,
            total_size: total_bytes,
            kernel,
        }
    }

    pub fn allocate(&self, size: usize) -> Option<AllocatedFrame> {
        let _size = size.next_power_of_two();

        todo!()
    }

    fn deallocate(&self, _size: usize) {}

    fn find_slot(size: usize) -> usize {
        assert!(size.is_power_of_two());

        let mut result = 0;
        let mut value = size;

        while value > 1 {
            value >>= 1;
            result += 1;
        }

        result
    }
}

pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();
