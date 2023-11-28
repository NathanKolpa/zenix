use crate::memory::alloc::{kernel_alloc, MemoryInfo, FRAME_ALLOC};
use crate::{arch, debug};
use bootloader::BootInfo;

/// Initialize and start the operating system.
pub fn init(boot_info: &'static BootInfo) {
    // We should first initialize architecture specific stuff before anything else.
    arch::init();

    debug_println!("Staring the Zenix operating system...");
    debug_println!("Architecture: {}", arch::NAME);
    debug_println!("Debug channel: {}", debug::DEBUG_CHANNEL);
    debug_println!("{}", MemoryInfo::from_memory_map(&boot_info.memory_map));

    // Initializing the heap is also very important to do first.
    // Even the frame allocator uses the heap!
    let kernel_size = kernel_alloc::init_heap();
    debug_println!(
        "Initialized the kernel heap with {} of backing.",
        &crate::util::display::ReadableSize::new(kernel_size)
    );

    FRAME_ALLOC.init_with(
        &boot_info.memory_map,
        boot_info.physical_memory_offset.into(),
    );
    // https://en.wikipedia.org/wiki/Non-blocking_linked_list
    // https://stackoverflow.com/questions/71316932/lock-free-stack-with-freelist-why-dont-the-next-pointers-need-to-be-atomic
    // https://os.phil-opp.com/paging-implementation/#map-the-complete-physical-memory
    // https://nfil.dev/kernel/rust/coding/rust-buddy-allocator/
    // https://github.com/red-rocket-computing/buddy-alloc/blob/master/doc/bitsquid-buddy-allocator-design.md
    // https://github.com/torvalds/linux/blob/master/include/linux/mm.h
    // https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html
    // https://www.geeksforgeeks.org/power-of-two-free-lists-allocators-kernal-memory-allocators/
    // https://wiki.osdev.org/Page_Frame_Allocation
    // https://www.youtube.com/watch?v=DRAHRJEAEso&t=838s
}
