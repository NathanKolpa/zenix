#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(doc_cfg)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
// TODO: remove when the kernel gets sufficiently complete.
#![allow(dead_code)]

extern crate alloc;

pub mod arch;
pub mod debug;
pub mod init;
pub mod memory;
#[cfg(test)]
pub mod testing;
pub mod util;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(_start);

/// The kernel panic handler.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_println!("{info}");
    arch::x86_64::halt_loop();
}

/// The kernel panic handler during testing.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::panic_handler(info)
}

/// The kernel entry point.
fn _start(boot_info: &'static BootInfo) -> ! {
    unsafe {
        init::init(boot_info);
    }

    #[cfg(test)]
    test_main();

    arch::x86_64::halt_loop();
}
