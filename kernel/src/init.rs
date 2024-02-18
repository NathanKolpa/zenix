use core::usize;

use bootinfo::BootInfo;

use crate::memory::{
    alloc::{kernel_alloc::KERNEL_ALLOC, MemoryInfo, FRAME_ALLOC},
    map::mapper::MemoryMapper,
};
use crate::{arch, debug};

fn print_info(boot_info: &BootInfo) {
    debug_println!("Staring the Zenix operating system...");
    debug_println!("Architecture: {}", arch::NAME);
    debug_println!("Debug channel: {}", debug::DEBUG_CHANNEL);
    if let Some(bootloader_name) = boot_info.bootloader_name() {
        debug_println!("Bootloader: {bootloader_name}");
    }
    debug_println!("{}", MemoryInfo::from_boot_info(boot_info));
}

/// Initialize and start the operating system.
///
/// # Safety
///
/// The argument `boot_info` should contain a valid memory map for the machine.
pub unsafe fn init(boot_info: &BootInfo) {
    // We should first initialize architecture specific stuff before anything else.
    arch::init();

    print_info(boot_info);

    // Initializing the heap is also very important to do first.
    // Even the frame allocator uses the heap!
    let heap = boot_info.usable_heap();
    KERNEL_ALLOC.add_backing(core::slice::from_raw_parts_mut(
        heap.start as *mut _,
        heap.size as usize,
    ));

    FRAME_ALLOC.init_with(
        boot_info.usable_memory(),
        boot_info.physycal_memory_offset(),
    );

    let mut root_mapper = MemoryMapper::from_active_page(boot_info.physycal_memory_offset());
    root_mapper.share_all();

    debug_println!(
        "{}",
        root_mapper.tree_display(0usize.into(), 1024 * 1024 * 10, None)
    );

    debug_println!("Graceull exit");
}
