#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(panic_info_message)]

mod bump_memory;
mod gdt;
mod long_mode;
mod multiboot;
mod paging;
mod vga;

use core::{
    arch::{asm, global_asm},
    u64, u8,
};

use essentials::address::VirtualAddress;

use crate::{
    bump_memory::BumpMemory, gdt::setup_gdt_table, long_mode::*, multiboot::MultibootInfo,
    paging::setup_paging,
};

global_asm!(include_str!("boot.s"));

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let msg = info
        .message()
        .and_then(|a| a.as_str())
        .unwrap_or("Uknown error");

    vga::set_fail_msg(msg);

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

extern "C" {
    static BUMP_MEMORY_START: u8;
    static BUMP_MEMORY_END: u8;
}

#[no_mangle]
pub extern "C" fn main(multiboot_magic_arg: u32, multiboot_info_addr: u32) {
    vga::clear_screen();
    vga::set_running_msg();

    if multiboot_magic_arg != 0x2BADB002 {
        vga::set_fail_msg("Multiboot magic value in EAX does not equal 0x2BADB002.");
        return;
    }

    if !cpuid_supported() {
        vga::set_fail_msg("CPUID not supported by your processor.");
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

    let info_ptr = multiboot_info_addr as *mut MultibootInfo;
    let info = unsafe { &mut *info_ptr };

    let Some(kernel_module) = info.take_first_mod() else {
        vga::set_fail_msg("No modules loaded. Have you configured your bootloader correctly?");
        return;
    };

    let Some(mmap) = info.mmap() else {
        vga::set_fail_msg("Multiboot Info does not contain the mmap_* fields.");
        return;
    };

    let mut bump_memory = unsafe {
        BumpMemory::new(
            VirtualAddress::from(&BUMP_MEMORY_START as *const _),
            VirtualAddress::from(&BUMP_MEMORY_END as *const _),
        )
    };

    let (l4_page_table, entry_point) = match setup_paging(&mut bump_memory, mmap, kernel_module) {
        Ok(t) => t,
        Err(e) => {
            vga::set_fail_msg(e.as_str());
            return;
        }
    };

    let gdt_table = setup_gdt_table(&mut bump_memory);

    unsafe {
        enter_long_mode(l4_page_table, &gdt_table);
    }

    vga::set_success_msg();

    unsafe { call_kernel_main(entry_point) };

    vga::set_fail_msg("Unexpectedly returned from kernel_main");
}

#[no_mangle]
#[inline(never)]
unsafe fn call_kernel_main(entry: u64) {
    // todo: reset the stack to the top
    asm!(
    "and esp, 0xffffff00",
    "push 0",
    "push {entry_point:e}",
    entry_point = in(reg) entry as u32
    );
    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
    asm!(
        ".code64",
        "call rax",
        in("rax") entry as u32
    )
}
