//! A physical frame allocator implementation (this is where **all** the memory comes from).
//!
//! The allocator is implemented
//! using the [Buddy Allocation Algorithm](https://www.youtube.com/watch?v=DRAHRJEAEso).
//! There are many flavours of the Buddy System, the [`FrameAllocator`] uses a version that the
//! [the Linux kernel also uses](https://www.kernel.org/doc/gorman/html/understand/understand009.html).
//!
//! # Heap usage
//!
//! Ironically, memory is needed to keep track of memory.
//! Because the kernel heap does not dynamically allocate frames it is perfectly fine for this module to use.
//! The heap usage within this module only consists of allocations at initialization, without resizes or deletes
//! but can vary in size based on the machine's memory.
//!
//! # Overview
//!
//! The memory that is available is not represented as a single flat line,
//! instead the bootloader gives us a list of usable regions.
//! Each region gets its own "heap", this is called a [`Zone`]
//! ([the same name that linux uses](https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html)).
//!
//! Each zone keeps track of a list of [`Level`]s where each index of a level corresponds to the order of magnitude of which that level manages.
//! A level consists of a (double-linked) [freelist](https://en.wikipedia.org/wiki/Free_list) and a [`crate::util::Bitmap`].
//! The freelist uses the global memory offset passed by the bootloader in order to read and write the previous and next pointer into the free blocks of memory.
//! The bitmap keeps track of which blocks are used and unused.
//! This becomes necessary when coalescing blocks of memory and figuring out the size of blocks when deallocating.
//!
//!
//! For more information on specific operation see: [`FrameAllocator`].

use alloc::vec::Vec;

use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

use zone::Zone;

#[allow(unused_imports)] // Used the module's doc.
use level::Level;

use essentials::address::PhysicalAddress;
use essentials::PanicOnce;

mod level;
mod zone;

const MIN_ORDER: u8 = 12; // 4Kib granularity.

pub struct FrameAllocator {
    zones: PanicOnce<Vec<Zone>>, // TODO: add this to the eternal alloc
}

impl FrameAllocator {
    const fn new() -> Self {
        Self {
            zones: PanicOnce::new(),
        }
    }

    /// Initialize the allocator with a memory map.
    ///
    /// # Overview
    ///
    /// The algorithm requires that all blocks are a power of two.
    /// But for region sizes, this is almost never the case.
    /// A naive solution to this problem would be to round down to the previous power of two.
    /// Lets say we have 119 Mib (an actual region size from qemu),
    /// then rounding down to 64 Mib would cost us
    /// almost half our memory!
    ///
    /// This problem can be solved by allowing incomplete blocks:
    /// First the region size is rounded up to the next nearest power of two
    /// so that 119 Mib would become 128 MiB.
    /// Then,
    /// for each level we take a piece of usable memory
    /// and make it available by adding it to the level's free list.
    /// And avoid to coalescing incomplete blocks when deallocating,
    /// each link listed block's buddy is marked as allocated.
    /// If a piece memory doesnt' fit in a level, we mark it as allocated and move on.
    /// We repeat the above steps until there is no more usable memory left.
    ///
    /// An example of how 344.0 KiB looks in a zone when initialized:
    /// ```txt
    /// 512 KiB:          *
    ///                 .` `.
    ///                /     \
    /// 256 KiB:      @       *
    ///             /  \     / \
    /// 128 KiB:   .    .   *   .
    ///           /|   /|  /|  /|
    /// 64 KiB : . .  . . @ * . .
    ///
    /// * = Marked as allocated in the level's bitmap.
    /// @ = Added in the the level's free list.
    /// . = Neither in the level's bitmap nor the free list.
    /// ```
    ///
    /// # Panics
    ///
    /// Calling this function more than once will result in a panic.
    ///
    /// # Safety
    ///
    /// The `map` and `physical_memory_offset` are assumed to be valid for the machine.
    pub unsafe fn init_with(&self, map: &MemoryRegions, physical_memory_offset: usize) {
        let usable_regions = map
            .iter()
            .filter(|x| x.kind == MemoryRegionKind::Usable)
            .filter(|x| (x.end - x.start) >= 2u64.pow(MIN_ORDER as u32))
            .map(|r| Zone::new((r.start).into(), (r.end).into(), physical_memory_offset));

        let mut zones: Vec<_> = usable_regions.collect();

        // sorting by size (descending) will make sure that we have the highest success chance first
        zones.sort_by_key(|b| core::cmp::Reverse(b.size()));

        self.zones.initialize_with(zones);
    }

    /// Allocate a block of physical memory.
    ///
    /// # Overview
    ///
    /// The size of the request is rounded up to the next nearest power of two.
    /// For example, if a request for 5000 bytes is made, the allocation is rounded up to 8192 bytes.
    /// With `MIN_ORDER` set to 12 (4kib minimum), this corresponds to level 1 (starting from 0).
    ///
    /// The corresponding level is determined based on the rounded-up size.
    /// If it's possible to pop a block of memory from the level's freelist, it is marked as used in the bitmap,
    /// and the request can be satisfied using (the address of) this block.
    /// If there is no available block of memory in the freelist, a recursive attempt is made to allocate a block in the level above.
    /// When the level above returns a block (exactly twice the size needed),
    /// the block is split in half. The first half is added to the freelist, and the second half is marked as used in the bitmap.
    /// Subsequently, the request is fulfilled using the second half.
    ///
    /// # Panics
    ///
    /// This function will panic when it is called before [`FrameAllocator::init_with`]
    ///
    /// # Returns
    ///
    /// When the request could be satisfied the function returns `Some` containing a tuple with the following elements:
    /// - The physical address of the start of the newly allocated block.
    /// - The size of the allocated block. This is guaranteed to be at least the size of the `size` argument or larger.
    pub fn allocate(&self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let size = size.next_power_of_two();
        self.zones.iter().find_map(|zone| zone.allocate(size))
    }

    /// Allocate a block of physical memory with every byte set to zero.
    ///
    /// This preforms the exact same operation as [`allocate`]. However, at the end it writes zero
    /// before returing the new block of memory.
    ///
    /// # Panics
    ///
    /// This function will panic when it is called before [`FrameAllocator::init_with`]
    ///
    /// # Returns
    ///
    /// When the request could be satisfied the function returns `Some` containing a tuple with the following elements:
    /// - The physical address of the start of the newly allocated block.
    /// - The size of the allocated block. This is guaranteed to be at least the size of the `size` argument or larger.
    pub fn allocate_zeroed(&self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let size = size.next_power_of_two();
        self.zones
            .iter()
            .find_map(|zone| zone.allocate_zeroed(size))
    }

    /// Deallocate a block of physical memory.
    ///
    /// # Overview
    ///
    /// Before its possible to start the de-allocation process we need to figure out the size of the block of memory we want to free.
    /// This can be done by checking for each level (from small to large) if the block is marked as used.
    /// If the block is used, then the level's size is our de-allocation size.
    ///
    /// Using the size, we get the corresponding level.
    /// In the level's bitmap we mark the block of memory as unused.
    /// If the the block's buddy is also unused, we can coalesce the two blocks so in the future larger requests can be satisfied.
    /// We achieve this by removing the buddy block from the freelist.
    /// Then we recursively deallocate for the next level.
    /// If it is not possible to coalesce a block we add it to the level's freelist.
    ///
    /// # Panics
    ///
    /// This function will panic when it is called before [`FrameAllocator::init_with`]
    ///
    /// Passing an invalid `addr` argument can sometimes result in a panic, however this won't always be the case.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - `addr` must denote a block of memory currently allocated via this allocator,
    /// - The underlying memory located at `addr` is not read of written after calling this function.

    pub unsafe fn deallocate(&self, addr: PhysicalAddress) {
        let zone = self
            .zones
            .iter()
            .find(|zone| zone.contains(addr))
            .expect("addr should be within a zone's bounds");

        zone.deallocate(addr);
    }

    /// Get the total amount of available memory.
    ///
    /// Note: this does not mean that it is possible to allocate a block of memory of the returned size.
    /// This is because the memory may to be contiguous.
    pub fn available(&self) -> usize {
        self.zones.iter().map(|zone| zone.available()).sum()
    }

    /// Get the minimum allocation size.
    pub fn min_size(&self) -> usize {
        2usize.pow(MIN_ORDER as u32)
    }
}

pub static FRAME_ALLOC: FrameAllocator = FrameAllocator::new();

/// For these tests `crate::init::init()` must be called.
#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_alloc_and_dealloc_page() {
        let (addr, size) = FRAME_ALLOC.allocate(4096).unwrap();
        assert_eq!(size, 4096);
        unsafe {
            FRAME_ALLOC.deallocate(addr);
        }
    }

    #[test_case]
    fn test_alloc_more_then_available_number_should_fail() {
        let available = FRAME_ALLOC.available();
        let result = FRAME_ALLOC.allocate(available + 1);
        assert_eq!(result, None);
    }

    #[test_case]
    fn test_dealloc_should_allow_reuse_of_memory() {
        let (addr1, size1) = FRAME_ALLOC.allocate(FRAME_ALLOC.min_size()).unwrap();
        assert_eq!(size1, FRAME_ALLOC.min_size());
        unsafe {
            FRAME_ALLOC.deallocate(addr1);
        }

        let (addr2, size2) = FRAME_ALLOC.allocate(FRAME_ALLOC.min_size()).unwrap();
        assert_eq!(size1, size2);
        assert_eq!(addr1, addr2);
    }

    #[test_case]
    fn test_reuse_more_then_the_entire_memory_map() {
        let available = FRAME_ALLOC.available();
        let n_alloc = available / FRAME_ALLOC.min_size() * 2;

        for _ in 0..n_alloc {
            let (addr, _) = FRAME_ALLOC.allocate(FRAME_ALLOC.min_size()).unwrap();
            unsafe {
                FRAME_ALLOC.deallocate(addr);
            }
        }
    }

    #[test_case]
    fn test_align_alloc_with_pages() {
        let (addr, size) = FRAME_ALLOC.allocate(FRAME_ALLOC.min_size() + 1).unwrap();
        assert_eq!(size, FRAME_ALLOC.min_size() * 2);

        unsafe {
            FRAME_ALLOC.deallocate(addr);
        }
    }

    #[test_case]
    fn test_alloc_twice_should_not_overlap() {
        let (addr1, size1) = FRAME_ALLOC.allocate(FRAME_ALLOC.min_size()).unwrap();
        let (addr2, size2) = FRAME_ALLOC.allocate(FRAME_ALLOC.min_size()).unwrap();

        assert_eq!(size1, size2);
        assert!(addr1 + size1 <= addr2 || addr1 >= addr2 + size2);

        unsafe {
            FRAME_ALLOC.deallocate(addr1);
            FRAME_ALLOC.deallocate(addr2);
        }
    }
}
