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

use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(_start, config = &BOOTLOADER_CONFIG);

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
fn _start(boot_info: &'static mut BootInfo) -> ! {
    unsafe {
        init::init(boot_info);
    }

    #[cfg(test)]
    test_main();

    arch::x86_64::halt_loop();
}
