use bootinfo::BootInfo;

use crate::{
    memory::{
        alloc::{kernel_alloc::KERNEL_ALLOC, FRAME_ALLOC},
        map::MemoryMapper,
    },
    multitasking::{scheduler::LOWEST_PRIORITY, PROCESS_TABLE, SCHEDULER},
};

use crate::{arch, debug_println};

/// Initialize and start the operating system.
///
/// # Safety
///
/// The argument `boot_info` should contain a valid memory map for the machine.
pub unsafe fn init(boot_info: &BootInfo) {
    // Initializing the heap is also very important to do first.
    // Even the frame allocator uses the heap!
    let heap = boot_info.usable_heap();
    KERNEL_ALLOC.add_backing(core::slice::from_raw_parts_mut(
        heap.start as *mut _,
        heap.size as usize,
    ));

    debug_println!("KERNEL_HEAP initialized");

    // When frame alloc is initialized then memory mapping is possible.
    FRAME_ALLOC.init_with(
        boot_info.usable_memory(),
        boot_info.physycal_memory_offset(),
    );

    debug_println!("FRAME_ALLOC initialized");

    let mut kernel_mem = MemoryMapper::new_root_mapper(boot_info.physycal_memory_offset());

    arch::init(boot_info, &mut kernel_mem);

    kernel_mem.share_all();
    debug_println!("Kernel virtual memory shared");

    SCHEDULER.init();

    let kernel_tid = SCHEDULER
        .current_as_kernel_thread(LOWEST_PRIORITY)
        .expect("calling current_as_kernel_thread should never fail in init()");

    debug_println!("SCHEDULER initialized; kernel thread id: {kernel_tid}");

    PROCESS_TABLE.init();
    debug_println!("PROCESS_TABLE initialized");
}
