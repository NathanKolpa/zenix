use crate::memory::MemoryInfo;
use crate::util::PanicOnce;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// A frame allocator using the [Buddy Allocator Algorithm](https://en.wikipedia.org/wiki/Buddy_memory_allocation).
pub struct FrameAllocator {
    memory_map: PanicOnce<&'static MemoryMap>,
}

impl FrameAllocator {
    const fn new() -> Self {
        Self {
            memory_map: PanicOnce::new(),
        }
    }

    pub fn init_with(&self, map: &'static MemoryMap) {
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
}

pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();
