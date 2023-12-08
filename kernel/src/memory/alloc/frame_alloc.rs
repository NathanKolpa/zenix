//! A physical frame allocation.
//!
//! Allocation is implemented
//! using the [Buddy Allocation Algorithm](https://www.youtube.com/watch?v=DRAHRJEAEso).
//! There are many flavours of the Buddy System, the [`FrameAllocator`] uses a version that the
//! [the Linux kernel also uses](https://www.kernel.org/doc/gorman/html/understand/understand009.html).
//!
//! # Heap usage
//!
//! Ironically, memory is needed to keep track of memory.
//! Because the kernel heap does not dynamically allocate frames it is perfectly fine to use.
//! The heap usage within this module only consists of allocations at initialization without resizes or deletes
//! but can vary in size based on the machine's memory.
//!
//! # Overview
//!
//! The memory that is available is not represented as a single flat line,
//! instead the bootloader gives us a list of usable regions.
//! Each region gets its own "heap", we call this a [`Zone`]
//! ([the same name that linux uses](https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html)).
//!
//! Within each zone: TODO
//!
//! ### Initialization
//!
//! The algorithm requires that all blocks are a power of two.
//! For region sizes, this is almost never the case.
//! A naive solution to this problem is to round down to the previous power of two.
//! Lets say we have 119 Mib (an actual region size from qemu),
//! then rounding down to 64 Mib would cost us
//! almost half our memory!
//!
//! This problem can be solved by allowing incomplete blocks:
//! First the region size is rounded up to the next nearest power of two
//! so that 119 Mib would become 128 MiB.
//! Then,
//! for each level we take a piece of usable memory
//! and make it available by adding it to the level's free list.
//! And avoid to coalescing incomplete blocks when deallocating,
//! each link listed block's buddy is marked as allocated.
//! If a piece memory doesnt' fit in a level, we mark it as allocated and move on.
//! We repeat the above steps until there is no more usable memory left.
//!
//! An example of how 344.0 KiB looks in a zone when initialized:
//! ```text
//! 512 KiB:          *
//!                 .` `.
//!                /     \
//! 256 KiB:      @       *
//!             /  \     / \
//! 128 KiB:   .    .   *   .
//!           /|   /|  /|  /|
//! 64 KiB : . .  . . @ * . .
//!
//! * = Marked as allocated in the level's bitmap.
//! @ = Added in the the level's free list.
//! . = Neither in the bitmap or free list.
//! ```

//! ### Allocation
//!
//! We first round up the `size` of the request to the next nearest power of two.
//! Say a request of 5000 is made, the allocation is rounded up to 8192 bytes.
//! With `MIN_ORDER` set to 12 (4kib minimum), this corresponds to level 1 (starting from 0).
//!
//! TODO

use alloc::vec::Vec;

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use zone::Zone;

use crate::util::address::{PhysicalAddress, VirtualAddress};
use crate::util::PanicOnce;

mod level;
mod node;
mod zone;

const MIN_ORDER: u8 = 12; // 4Kib granularity.

pub struct FrameAllocator {
    areas: PanicOnce<Vec<Zone>>, // TODO: add this to the eternal alloc
}

impl FrameAllocator {
    const fn new() -> Self {
        Self {
            areas: PanicOnce::new(),
        }
    }

    /// Initialize the allocator with a memory map.
    ///
    /// # Safety
    ///
    /// The `map` and `physical_memory_offset` are assumed to be valid for the machine.
    pub unsafe fn init_with(&self, map: &MemoryMap, physical_memory_offset: usize) {
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

    pub fn allocate(&self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let _size = size.next_power_of_two();

        todo!()
    }

    pub unsafe fn deallocate(&self, _addr: PhysicalAddress) {}
}

pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();
