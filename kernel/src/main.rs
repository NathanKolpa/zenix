#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(doc_cfg)]
#![feature(abi_x86_interrupt)]
// TODO: remove when the kernel gets sufficiently complete.
#![allow(dead_code)]

mod arch;
mod debug;
mod init;
mod memory;
#[cfg(test)]
mod testing;
mod util;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(_start);

/// The kernel panic handler during testing
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("{info}");
    arch::x86_64::halt_loop();
}

/// The kernel panic handler.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::testing::panic_handler(info)
}

/// The kernel entry point.
fn _start(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    init::init(boot_info);

    arch::x86_64::halt_loop();
}
