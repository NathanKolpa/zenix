use alloc::vec;
use alloc::vec::Vec;

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use crate::util::address::{PhysicalAddress, VirtualAddress};
use crate::util::{Bitmap, FixedVec, PanicOnce};

const MAX_ORDER: usize = 24; // TODO: how much do we need?
const MIN_ORDER: usize = 12; // 4Kib granularity.

const ORDER_CAP: usize = MAX_ORDER - MIN_ORDER;

/// Same abstraction as a [Linux zone](https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html).
struct Zone {
    addr_start: PhysicalAddress,
    addr_end: PhysicalAddress,

    free_list: FixedVec<ORDER_CAP, Level>,
    physical_memory_offset: VirtualAddress,
}

impl Zone {
    pub fn new(
        addr_start: PhysicalAddress,
        addr_end: PhysicalAddress,
        physical_memory_offset: VirtualAddress,
    ) -> Self {
        assert!(addr_end > addr_start);
        assert!((addr_end - addr_start).as_usize() > 0);

        let mut free_list = FixedVec::new();

        for order in 0..MAX_ORDER {
            free_list.push(Level::empty(order as u8))
        }

        Self {
            addr_start,
            addr_end,
            free_list,
            physical_memory_offset,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let size = size.next_power_of_two();
        let _order = Self::order_of(size);

        todo!()
    }

    fn order_of(size: usize) -> u8 {
        assert!(size.is_power_of_two());

        let mut result = 0u8;
        let mut value = size;

        while value > 1 {
            value >>= 1;
            result += 1;
        }

        result
    }
}

struct FreeListNode {
    next: Option<PhysicalAddress>,
}

struct Level {
    free_list: Option<PhysicalAddress>,

    bitmap: Bitmap<Vec<u8>>, // TODO: add this to the eternal alloc
}

impl Level {
    pub fn empty(order: u8) -> Self {
        let bits = 2usize.pow((MAX_ORDER - order as usize) as u32);
        let cap = (bits + 8 - 1) / 8; // div ceil 8 the number of bits

        Self {
            free_list: None,
            bitmap: Bitmap::new(vec![0; cap]),
        }
    }
}

/// This is where **all** the memory comes from :^)
///
/// A frame allocator implemented
/// using the [Buddy Allocator Algorithm](https://www.youtube.com/watch?v=DRAHRJEAEso),
/// this is the same algorithm that [the Linux kernel uses](https://www.kernel.org/doc/gorman/html/understand/understand009.html).
///
/// # How it works
///
/// A (allocation) request goes as follows:
///
/// We first round up the `size` of the request to the next nearest power of two.
/// Say we request 5000, then the allocation returns a block of 8192 bytes.
/// With `MIN_ORDER` set to 12 (4kib minimum), this corresponds to level 1 (starting from 0).
///
/// Now we look into the free list of that level to find a matching block.
/// First translate the physical address to a virtual one in order to dereference a node.
/// If a node is found,
/// we remove it from the list and we can satisfy the request with its (physical) address.
/// If we cannot find a node,
/// we recurse one level above and split (if a node is found) the node into two parts.
/// We take one half and put it in the free list for the level below,
/// we use the other half for the previous call below.
pub struct FrameAllocator {
    areas: PanicOnce<Vec<Zone>>, // TODO: add this to the eternal alloc
}

impl FrameAllocator {
    const fn new() -> Self {
        Self {
            areas: PanicOnce::new(),
        }
    }

    pub fn init_with(&self, map: &MemoryMap, physical_memory_offset: VirtualAddress) {
        let usable_regions = map
            .iter()
            .filter(|x| x.region_type == MemoryRegionType::Usable)
            .map(|r| {
                Zone::new(
                    (r.range.start_frame_number * 4096).into(),
                    (r.range.end_frame_number * 4096).into(),
                    physical_memory_offset,
                )
            });

        self.areas.initialize_with(usable_regions.collect());
    }

    pub fn allocate(&self, size: usize) -> Option<PhysicalAddress> {
        let _size = size.next_power_of_two();

        todo!()
    }

    pub unsafe fn deallocate(&self, _addr: PhysicalAddress) {}
}

pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();
