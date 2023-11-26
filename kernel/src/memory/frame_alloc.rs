use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::memory::MemoryInfo;
use crate::util::address::PhysicalAddress;
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
    start: AtomicUsize,
}

impl<'a> FrameAllocator<'a> {
    const fn new() -> Self {
        Self {
            memory_map: PanicOnce::new(),
            slots: FixedVec::new(),
            start: AtomicUsize::new(0),
        }
    }

    pub fn init_with(&self, map: &'a MemoryMap) {
        self.memory_map.initialize_with(map);
    }

    pub fn info(&self) -> MemoryInfo {
        let bytes_allocated = self.start.load(Ordering::Relaxed);

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

    /// Allocate a chunk of physical memory without the ability to deallocate.
    pub fn allocate_eternally(&self, size: usize) -> Option<PhysicalAddress> {
        // TODO: check if the memory does not touch the free lists.
        let start = self.start.fetch_add(size, Ordering::SeqCst);

        let usable_regions = self
            .memory_map
            .iter()
            .filter(|region| region.region_type == MemoryRegionType::Usable);

        let mut current = start;

        for usable_region in usable_regions {
            let region_size = usable_region.range.end_addr() - usable_region.range.start_addr();

            if current + size > region_size as usize {
                current = current.saturating_sub(region_size as usize);
                continue;
            }

            return Some(PhysicalAddress::new(
                usable_region.range.start_addr() as usize + current,
            ));
        }

        None
    }
}

pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();

#[cfg(test)]
mod tests {
    use super::*;
    use bootloader::bootinfo::{FrameRange, MemoryRegion};

    #[test_case]
    fn test_allocate_eternally_all_memory() {
        let mut map = MemoryMap::new();
        map.add_region(MemoryRegion {
            range: FrameRange::new(0, 4096),
            region_type: MemoryRegionType::Usable,
        });

        let alloc = FrameAllocator::new();
        alloc.init_with(&map);

        assert_eq!(
            alloc.allocate_eternally(4095),
            Some(PhysicalAddress::new(0))
        );
        assert_eq!(
            alloc.allocate_eternally(1),
            Some(PhysicalAddress::new(4095))
        );
        assert_eq!(alloc.allocate_eternally(1), None);
    }

    #[test_case]
    fn test_allocate_eternally_in_next_chunk() {
        let mut map = MemoryMap::new();
        map.add_region(MemoryRegion {
            range: FrameRange::new(0, 4096),
            region_type: MemoryRegionType::Usable,
        });

        let second_chunk_start = 4096 * 10;

        map.add_region(MemoryRegion {
            range: FrameRange::new(second_chunk_start, second_chunk_start + 4096 * 5),
            region_type: MemoryRegionType::Usable,
        });

        let alloc = FrameAllocator::new();
        alloc.init_with(&map);
        assert_eq!(
            alloc.allocate_eternally(4097),
            Some(PhysicalAddress::from(second_chunk_start))
        );
    }
}
