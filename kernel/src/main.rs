#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(test)]
mod testing;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(_start);

/// The kernel panic handler during testing
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// The kernel panic handler.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::testing::panic_handler(info)
}

/// The kernel entry point.
fn _start(_boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    loop {}
}
