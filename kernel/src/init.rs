use bootinfo::BootInfo;
use x86_64::interrupt::enable_interrupts;

use crate::{
    info_print,
    memory::{
        alloc::{kernel_alloc::KERNEL_ALLOC, MemoryInfo, FRAME_ALLOC},
        map::mapper::MemoryMapper,
    },
    multitasking::{scheduler::LOWEST_PRIORITY, SCHEDULER},
};

use crate::{arch, debug_println, info_println};

fn print_info(boot_info: &BootInfo) {
    info_println!("Staring the Zenix operating system...");
    info_println!("Architecture: {}", arch::NAME);
    arch::print_info();
    info_println!("Debug channel: {}", crate::log::CHANNEL_NAME);
    if let Some(bootloader_name) = boot_info.bootloader_name() {
        info_println!("Bootloader: {bootloader_name}");
    }
    info_print!("{}", MemoryInfo::from_boot_info(boot_info));
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

    debug_println!("Kernel heap initialized");

    FRAME_ALLOC.init_with(
        boot_info.usable_memory(),
        boot_info.physycal_memory_offset(),
    );

    debug_println!("FRAME_ALLOC initialized");

    let mut root_mapper = MemoryMapper::new_root_mapper(boot_info.physycal_memory_offset());
    root_mapper.share_all();

    debug_println!("Virtual memory initialized");

    SCHEDULER.init();
    let kernel_tid = SCHEDULER
        .current_as_thread(LOWEST_PRIORITY)
        .expect("calling current_as_thread should never fail in init()");

    debug_println!("Scheduler initialized; kernel thread id: {kernel_tid}");

    debug_println!("Graceull exit");
    enable_interrupts();
}
