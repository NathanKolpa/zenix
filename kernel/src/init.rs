use core::usize;

use bootloader_api::BootInfo;

use crate::{arch, debug};
use crate::{
    memory::{
        alloc::{kernel_alloc, MemoryInfo, FRAME_ALLOC},
        map::mapper::{MemoryMapper, MemoryProperties},
    },
    util::address::VirtualAddress,
};

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
    debug_println!("{}", MemoryInfo::from_memory_map(&boot_info.memory_regions));

    let phys_mem_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("physical memory offset is passed by the bootloader")
        as usize;

    // Initializing the heap is also very important to do first.
    // Even the frame allocator uses the heap!
    let kernel_size = kernel_alloc::init_heap();
    debug_println!(
        "Initialized the kernel heap with {} of backing.",
        &crate::util::display::ReadableSize::new(kernel_size)
    );

    FRAME_ALLOC.init_with(&boot_info.memory_regions, phys_mem_offset);

    let mut root_mapper = MemoryMapper::from_active_page(phys_mem_offset);
    root_mapper.share_all();

    debug_println!("{}", root_mapper.tree_display(Some(0)));
}
