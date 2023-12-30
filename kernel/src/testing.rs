//! A test harness.

use core::panic::PanicInfo;

use crate::arch::x86_64::device::qemu::{ExitCode, QEMU_DEVICE};
use crate::{debug_print, debug_println};

pub trait TestCase {
    fn run(&self, test_number: usize, test_count: usize);
}

const TEST_NAME_ALIGN_TO: usize = 100;

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self, test_number: usize, test_count: usize) {
        let test_name = core::any::type_name::<T>();
        let padding = TEST_NAME_ALIGN_TO.saturating_sub(test_name.len());

        debug_print!(
            "  ({test_number}/{test_count}) =>\t{test_name}...{: <1$}",
            "",
            padding
        );
        self();
        debug_println!("[ok]");
    }
}

pub fn runner(tests: &[&dyn TestCase]) {
    debug_println!(
        "Running {} unit tests in post-initialization environment:",
        tests.len()
    );

    for (i, test) in tests.iter().enumerate() {
        test.run(i + 1, tests.len());
    }

    debug_println!("All unit tests completed successfully, shutting down...");

    QEMU_DEVICE.lock().exit(ExitCode::Success);
}
pub fn panic_handler(info: &PanicInfo) -> ! {
    debug_println!("[failed]");
    debug_println!("Error: {}", info);
    debug_println!("Unit test failed, shutting down...");
    QEMU_DEVICE.lock().exit(ExitCode::Failed);
}
