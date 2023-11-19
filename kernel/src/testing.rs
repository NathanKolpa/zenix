//! A test harness.

use core::panic::PanicInfo;

use crate::arch::x86_64::device::qemu::{ExitCode, QEMU_DEVICE};
use crate::{debug_print, debug_println};

pub trait TestCase {
    fn run(&self);
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        debug_print!("{}...\t", core::any::type_name::<T>());
        self();
        debug_println!("[ok]");
    }
}

pub fn runner(tests: &[&dyn TestCase]) {
    debug_println!("Running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    QEMU_DEVICE.lock().exit(ExitCode::Success);
}
pub fn panic_handler(info: &PanicInfo) -> ! {
    debug_println!("[failed]\n");
    debug_println!("Error: {}\n", info);
    QEMU_DEVICE.lock().exit(ExitCode::Failed);
}
