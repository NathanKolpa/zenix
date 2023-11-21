use crate::{arch, debug};

/// Initialize and start the operating system.
pub fn init() {
    // We should first initialize architecture specific stuff before anything else.
    arch::init();

    debug_println!("Staring the Zenix operating system...");
    debug_println!("Architecture: {}", arch::NAME);
    debug_println!("Debug channel: {}", debug::DEBUG_CHANNEL);
}
