#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(doc_cfg)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
// TODO: remove when the kernel gets sufficiently complete.
#![allow(dead_code)]

extern crate alloc;

pub mod arch;
pub mod init;
pub mod interface;
pub mod log;
pub mod memory;
pub mod multitasking;

#[cfg(test)]
pub mod testing;
mod utils;

use core::panic::PanicInfo;

use bootinfo::{BootInfo, BootInfoData};

/// The kernel panic handler.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error_println!("{info}");
    x86_64::halt_loop();
}

/// The kernel panic handler during testing.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::panic_handler(info)
}

/// The kernel entry point.
#[no_mangle]
unsafe extern "C" fn kernel_main(boot_info_ptr: *const BootInfoData) -> ! {
    let boot_info = BootInfo::deref_ptr(boot_info_ptr);

    init::init(&boot_info);

    #[cfg(test)]
    test_main();

    run()
}

fn run() -> ! {
    use x86_64::interrupt::*;

    loop {
        disable_interrupts();

        // Do work mabye..

        enable_interrupts_and_halt();
    }
}
