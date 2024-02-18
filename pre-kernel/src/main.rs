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

use crate::{
    boot_info::{finalize_boot_info, setup_boot_info},
    bump_memory::BumpMemory,
    gdt::setup_gdt_table,
    long_mode::*,
    multiboot::{MultibootInfo, MULTIBOOT_MAGIC},
    paging::{map_kernel, setup_paging, PagingSetupError},
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

enum PreKernelError {
    InvalidMulitbootMagic,
    CpuidNotsupported,
    ExtendedCpuidNotSupported,
    LongModeNotSupported,
    NoModulesLoaded,
    NoMemoryMapInfo,
    FailedToSetupPaging(PagingSetupError),
    FailedToMapKernel(PagingSetupError),
    UnexpectedReturn,
}

impl PreKernelError {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreKernelError::InvalidMulitbootMagic => {
                "Multiboot magic value in EAX does not equal 0x2BADB002."
            }
            PreKernelError::CpuidNotsupported => "CPUID not supported by your processor.",
            PreKernelError::ExtendedCpuidNotSupported => {
                "Extended CPUID not supported by your processor."
            }
            PreKernelError::LongModeNotSupported => "Long Mode not supported by your processor.",
            PreKernelError::NoModulesLoaded => {
                "No modules loaded. Have you configured your bootloader correctly?"
            }
            PreKernelError::NoMemoryMapInfo => "Multiboot Info does not contain the mmap_* fields.",
            PreKernelError::FailedToSetupPaging(e) => e.as_str(),
            PreKernelError::FailedToMapKernel(e) => e.as_str(),
            PreKernelError::UnexpectedReturn => "Unexpected return from kernel_main",
        }
    }
}

fn run(multiboot_magic_arg: u32, multiboot_info_addr: u32) -> Result<(), PreKernelError> {
    if multiboot_magic_arg != MULTIBOOT_MAGIC {
        return Err(PreKernelError::InvalidMulitbootMagic);
    }

    if !cpuid_supported() {
        return Err(PreKernelError::CpuidNotsupported);
    }

    if !extended_cpuid_supported() {
        return Err(PreKernelError::ExtendedCpuidNotSupported);
    }

    if !long_mode_supported() {
        return Err(PreKernelError::LongModeNotSupported);
    }

    let info_ptr = multiboot_info_addr as *mut MultibootInfo;
    let info = unsafe { &mut *info_ptr };

    let kernel_module = info
        .take_first_mod()
        .ok_or(PreKernelError::NoModulesLoaded)?;

    // It's important to put the kernel_module_region on the stack.
    // Because `map_kernel` needs access to this information and will result in a page fault if it
    // is implicitly unmapped after `enable_paging`.
    let kernel_module_region = kernel_module.as_info_region();

    let mmap = info.mmap().ok_or(PreKernelError::NoMemoryMapInfo)?;

    let mut bump_memory = unsafe { BumpMemory::new_from_linker() };

    let gdt_table = setup_gdt_table(&mut bump_memory);

    // Create the kernel boot info before enabling paging, because the mulitboot info can't be accessed after
    // paging doing so.
    let kernel_boot_info = setup_boot_info(&mut bump_memory, mmap, kernel_module_region, info);

    let l4_page_table = setup_paging(&mut bump_memory, mmap.clone(), &kernel_module)
        .map_err(PreKernelError::FailedToSetupPaging)?;

    // Enable paging, this is required for the `map_kernel` function.
    // From now on, multi boot information is not mapped anymore.
    unsafe {
        enable_paging(l4_page_table as *const _ as u32);
    }

    let kernel_entry_point = unsafe {
        map_kernel(&mut bump_memory, &kernel_module, l4_page_table)
            .map_err(PreKernelError::FailedToMapKernel)?
    };

    // Set the final parameters of the kernel boot info because the kernel needs to know how much
    // memory is left over in the bump memory.
    finalize_boot_info(bump_memory, kernel_boot_info);

    unsafe {
        enter_long_mode(&gdt_table);
    }

    vga::set_success_msg();

    unsafe { call_kernel_main(kernel_entry_point, kernel_boot_info as *const _ as u64) };

    Err(PreKernelError::UnexpectedReturn)
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

        // refresh cr3
        "mov rdi, cr3",
        "mov cr3, rdi",

        "pop rdi",
        "pop rax",
        "call rax",
        out("rax") _,
        out("rdi") _
    )
}

#[no_mangle]
pub extern "C" fn main(multiboot_magic_arg: u32, multiboot_info_addr: u32) {
    vga::clear_screen();
    vga::set_running_msg();

    if let Err(err) = run(multiboot_magic_arg, multiboot_info_addr) {
        vga::set_fail_msg(err.as_str());
        return;
    }
}
