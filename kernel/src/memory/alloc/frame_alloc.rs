use alloc::vec;
use alloc::vec::Vec;
use core::cmp::{max, min};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use crate::util::address::{PhysicalAddress, VirtualAddress};
use crate::util::display::ReadableSize;
use crate::util::{Bitmap, PanicOnce};

const MIN_ORDER: u8 = 12; // 4Kib granularity.

/// Same abstraction as a [Linux zone](https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html).
struct Zone {
    addr_start: PhysicalAddress,
    addr_end: PhysicalAddress,

    free_list: Vec<Level>, // TODO: add this to the eternal alloc
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

        let total_size = (addr_end - addr_start).as_usize();
        let largest_order = Self::order_of_down(total_size);

        assert!(largest_order >= MIN_ORDER);

        let level_count = (largest_order - MIN_ORDER) as usize;

        let mut free_list = Vec::with_capacity(level_count + 1);

        for order in MIN_ORDER..=largest_order {
            debug_println!(
                "{} {order} {}",
                ReadableSize::new(total_size),
                ReadableSize::new(2usize.pow(order as u32))
            );
            free_list.push(Level::new(order, largest_order))
        }

        // TODO: figure out what to do with wasted space.
        // idea,
        // say we have 119 Mib
        // we dont want to downgrade the tree to only 64 MiB.
        // Instead we round up to 128 Mib, then we initialize with:
        // level 128 Mib => allocated.
        // Level 64 Mib (left) => allocated | add that chunk to the free list | we have 55 Mib left.
        // Level 32 Mib (left) => allocated | add that chunk to the free list | we have 23 Mib Left.
        // so on and so forth..

        Self {
            addr_start,
            addr_end,
            free_list,
            physical_memory_offset,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let size = size.next_power_of_two();
        let _order = Self::order_of_up(size);

        todo!()
    }

    fn order_of_up(size: usize) -> u8 {
        assert!(size.is_power_of_two());

        let mut result = 0u8;
        let mut value = size;

        while value > 1 {
            value >>= 1;
            result += 1;
        }

        result
    }

    fn order_of_down(size: usize) -> u8 {
        if size <= 1 {
            return 0;
        }

        let mut power = 1;
        let mut order = 0;

        while power * 2 <= size {
            power <<= 1;
            order += 1;
        }

        order
    }
}
struct FreeListNode {
    next: Option<PhysicalAddress>,
}

struct Level {
    free_list: Option<PhysicalAddress>,
    order: u8,
    bitmap: Bitmap<Vec<u8>>, // TODO: add this to the eternal alloc
}

impl Level {
    pub fn new(order: u8, largest_order: u8) -> Self {
        let bits = 2usize.pow(((largest_order - order) as usize) as u32);
        debug_println!("bits: {bits}");
        let bytes = (bits + 8 - 1) / 8; // div ceil 8 the number of bits

        Self {
            free_list: None,
            order,
            bitmap: Bitmap::new(vec![0; bytes]),
        }
    }

    pub fn size(&self) -> usize {
        2usize.pow(self.order as u32)
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
