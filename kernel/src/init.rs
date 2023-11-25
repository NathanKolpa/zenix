use crate::memory::FRAME_ALLOC;
use crate::{arch, debug};
use bootloader::BootInfo;

/// Initialize and start the operating system.
pub fn init(boot_info: &'static BootInfo) {
    // We should first initialize architecture specific stuff before anything else.
    arch::init();

    debug_println!("Staring the Zenix operating system...");
    debug_println!("Architecture: {}", arch::NAME);
    debug_println!("Debug channel: {}", debug::DEBUG_CHANNEL);

    FRAME_ALLOC.init_with(&boot_info.memory_map);
    debug_println!("{}", FRAME_ALLOC.info());
}
