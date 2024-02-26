use bootinfo::BootInfo;
use x86_64::interrupt::enable_interrupts;

use crate::memory::{
    alloc::{kernel_alloc::KERNEL_ALLOC, MemoryInfo, FRAME_ALLOC},
    map::mapper::MemoryMapper,
};

use crate::{arch, debug_println, info_println};

fn print_info(boot_info: &BootInfo) {
    info_println!("Staring the Zenix operating system...");
    info_println!("Architecture: {}", arch::NAME);
    info_println!("Debug channel: {}", crate::log::CHANNEL_NAME);
    if let Some(bootloader_name) = boot_info.bootloader_name() {
        info_println!("Bootloader: {bootloader_name}");
    }
    info_println!("{}", MemoryInfo::from_boot_info(boot_info));
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

    let _root_mapper = MemoryMapper::new_root_mapper(boot_info.physycal_memory_offset());

    enable_interrupts();
    debug_println!("Graceull exit");
}
