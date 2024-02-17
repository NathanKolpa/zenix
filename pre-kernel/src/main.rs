#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(panic_info_message)]

mod boot_info;
mod bump_memory;
mod gdt;
mod long_mode;
mod multiboot;
mod paging;
mod regions;
mod vga;

use core::arch::{asm, global_asm};

use essentials::address::VirtualAddress;

use crate::{
    boot_info::setup_boot_info,
    bump_memory::BumpMemory,
    gdt::setup_gdt_table,
    long_mode::*,
    multiboot::{MultibootInfo, MULTIBOOT_MAGIC},
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

#[no_mangle]
pub extern "C" fn main(multiboot_magic_arg: u32, multiboot_info_addr: u32) {
    vga::clear_screen();
    vga::set_running_msg();

    if multiboot_magic_arg != MULTIBOOT_MAGIC {
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

    let kernel_module_region = kernel_module.as_info_region();

    let Some(mmap) = info.mmap() else {
        vga::set_fail_msg("Multiboot Info does not contain the mmap_* fields.");
        return;
    };

    let mut bump_memory = unsafe { BumpMemory::new_from_linker() };

    let (l4_page_table, entry_point) =
        match setup_paging(&mut bump_memory, mmap.clone(), kernel_module) {
            Ok(t) => t,
            Err(e) => {
                vga::set_fail_msg(e.as_str());
                return;
            }
        };

    let gdt_table = setup_gdt_table(&mut bump_memory);

    let kernel_boot_info = setup_boot_info(bump_memory, mmap, kernel_module_region, info);

    unsafe {
        enter_long_mode(l4_page_table, &gdt_table);
    }

    vga::set_success_msg();

    unsafe { call_kernel_main(entry_point, kernel_boot_info) };

    vga::set_fail_msg("Unexpectedly returned from kernel_main");
}

#[no_mangle]
#[inline(never)]
unsafe fn call_kernel_main(entry: u64, kernel_boot_info: u64) {
    let stack_end = unsafe { &regions::STACK_END as *const _ as u32 };

    asm!(
        "mov esp, {stack_end:e}",
        "mov ebp, esp",
        "push 0",
        "push {entry_point:e}",
        "push 0",
        "push {kernel_boot_info:e}",
        entry_point = in(reg) entry as u32,
        kernel_boot_info = in(reg) kernel_boot_info as u32,
        stack_end = in(reg) stack_end
    );
    asm!("ljmp $0x8, $2f", "2:", options(att_syntax));
    asm!(
        ".code64",
        "pop rdi",
        "pop rax",
        "call rax",
        out("rax") _,
        out("rdi") _
    )
}
