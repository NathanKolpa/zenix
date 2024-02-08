use core::arch::asm;

use x86_64::segmentation::GlobalDescriptorTable;

use crate::gdt::InitialGdt;

pub fn cpuid_supported() -> bool {
    const CHECK_BIT: u32 = 1 << 21;
    let modified: u32;
    let original: u32;

    unsafe {
        asm!(
        "pushfd",
        "pop eax",

        "mov ecx, eax",

        "xor eax, {check_bit}",

        "push eax",
        "popfd",

        "pushfd",
        "pop eax",

        "push ecx",
        "popfd",

        out("ecx") original,
        out("eax") modified,
        check_bit = const CHECK_BIT
        );
    }

    modified != original
}

pub fn extended_cpuid_supported() -> bool {
    let mut output: u32 = 0x80000000;

    unsafe {
        asm!(
        "cpuid",
        inout("eax") output => output,
        out("ebx") _,
        out("ecx") _,
        out("edx") _,
        options(nomem, nostack, preserves_flags)
        )
    }

    output > 0x80000001
}

pub fn long_mode_supported() -> bool {
    let input: u32 = 0x80000000;
    let output: u32;

    unsafe {
        asm!(
        "cpuid",
        in("eax") input,
        lateout("ebx") _,
        lateout("ecx") _,
        lateout("edx") output,
        options(nomem, nostack, preserves_flags)
        )
    }

    (output & 1 << 29) != 0
}

#[no_mangle] // make debugging easier
#[inline(never)]
pub unsafe fn enter_long_mode(l4_page_table: u32, gdt_table: InitialGdt) {
    const PAE_FLAG: u32 = 1 << 5;
    const EFER_MSR: u32 = 0xC0000080;
    const LM_BIT: u32 = 1 << 8;
    const PG_BIT: u32 = 1 << 31;

    asm!(
        // load the l4 page table
        "mov cr3, {l4_page_table}",

        // enable PAE-paging by setting the PAE-bit in the fourth control register:
        "mov eax, cr4",
        "or eax, {PAE_FLAG}",
        "mov cr4, eax",
        // Now, paging is set up, but it isn't enabled yet.

        // There's not much left to do. We should set the long mode bit in the EFER MSR and then we should enable paging and protected mode and then we are in compatibility mode (which is part of long mode.
        "rdmsr",
        "or eax, {LM_BIT}",
        "wrmsr",

        // Enabling paging
        "mov eax, cr0",
        "or eax, {PG_BIT}",
        "mov cr0, eax",
        // Now we're in compatibility mode.

         l4_page_table = in(reg) l4_page_table,
         out("eax") _,
         in("ecx") EFER_MSR,
         PAE_FLAG = const PAE_FLAG,
         LM_BIT = const LM_BIT,
         PG_BIT = const PG_BIT,
    );
}
