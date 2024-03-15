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
#![feature(asm_const)]
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

use x86_64::{halt_loop, interrupt::*};

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

fn print_info(boot_info: &BootInfo) {
    info_println!("Architecture: {}", arch::NAME);
    info_println!("Debug channel: {}", crate::log::CHANNEL_NAME);
    if let Some(bootloader_name) = boot_info.bootloader_name() {
        info_println!("Bootloader: {bootloader_name}");
    }
    info_print!("{}", memory::alloc::MemoryInfo::from_boot_info(boot_info));
}

/// The kernel entry point.
#[no_mangle]
unsafe extern "C" fn kernel_main(boot_info_ptr: *const BootInfoData) -> ! {
    let boot_info = BootInfo::deref_ptr(boot_info_ptr);

    info_println!("Staring the Zenix operating system...");
    print_info(&boot_info);

    init::init(&boot_info);
    info_println!("System initialization complete");

    enable_interrupts();

    #[cfg(test)]
    test_main();

    run()
}

fn run() -> ! {
    halt_loop()
}
