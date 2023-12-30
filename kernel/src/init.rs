use bootloader::BootInfo;

use crate::memory::alloc::{kernel_alloc, MemoryInfo, FRAME_ALLOC};
use crate::{arch, debug};

/// Initialize and start the operating system.
///
/// # Safety
///
/// The argument `boot_info` should contain a valid memory map for the machine.
pub unsafe fn init(boot_info: &'static BootInfo) {
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
        boot_info.physical_memory_offset as usize,
    );
}
