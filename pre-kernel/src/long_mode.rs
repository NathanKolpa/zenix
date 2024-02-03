use core::arch::asm;

pub fn extended_cpuid_supported() -> bool {
    let mut output: u32 = 0x80000000;

    unsafe {
        asm!(
        "cpuid",
        inout("eax") output => output,
        lateout("ebx") _,
        lateout("ecx") _,
        lateout("edx") _,
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
