use core::arch::asm;

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
