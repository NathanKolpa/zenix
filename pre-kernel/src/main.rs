#![no_std]
#![no_main]
#![feature(asm_const)]
mod bump_memory;
mod long_mode;
mod multiboot;
mod vga;

use core::{
    arch::{asm, global_asm},
    fmt::Write,
    u32,
};

use bootinfo::BootInfo;
use essentials::address::VirtualAddress;
use x86_64::device::uart_16550::Uart16550;

use crate::{bump_memory::BumpMemory, long_mode::*, multiboot::MultibootInfo};

global_asm!(include_str!("boot.s"));

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("hlt");
    }
    loop {}
}

extern "C" {
    static BUMP_MEMORY_START: u8;
}

#[no_mangle]
pub extern "C" fn main(multiboot_magic_arg: u32, multiboot_info_addr: u32) {
    vga::clear_screen();
    vga::set_running_msg();

    if multiboot_magic_arg != 0x2BADB002 {
        vga::set_fail_msg("Multiboot magic value in EAX does not equal 0x2BADB002.");
        return;
    }

    if !extended_cpuid_supported() {
        vga::set_fail_msg("Extended CPUID not supported by your processor.");
        return;
    }

    if !long_mode_supported() {
        vga::set_fail_msg("Long Mode not supported by your processor.");
        return;
    }

    let info_ptr = multiboot_info_addr as *const MultibootInfo;
    let info = unsafe { &*info_ptr };

    let mut serial = unsafe { Uart16550::new_and_init(0x3F8) };

    let Some(mmap) = info.mmap() else {
        vga::set_fail_msg("Multiboot Info does not contain the mmap_* fields.");
        return;
    };

    for entry in mmap {
        writeln!(
            &mut serial,
            "Entry {} {} {}",
            entry.addr(),
            entry.len(),
            entry.kind(),
        );
    }

    let mut bump_memory =
        unsafe { BumpMemory::new(VirtualAddress::from(&BUMP_MEMORY_START as *const _)) };
}
