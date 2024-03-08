use core::arch::asm;

use essentials::address::PhysicalAddress;

/// WRMSR — Write to Model Specific Register
///
/// From [felixcloutier](https://www.felixcloutier.com/x86/wrmsr):
///
/// > Writes the contents of registers EDX:EAX into the 64-bit model specific register (MSR) specified in the ECX register.
/// > (On processors that support the Intel 64 architecture, the high-order 32 bits of RCX are ignored.)
/// > The contents of the EDX register are copied to high-order 32 bits of the selected MSR and the contents of the EAX register are copied to low-order 32 bits of the MSR.
/// > (On processors that support the Intel 64 architecture, the high-order 32 bits of each of RAX and RDX are ignored.)
/// > Undefined or reserved bits in an MSR should be set to values previously read.
///
/// # Safety
///
/// Passing invalid paramters into the register can cause UB.
pub unsafe fn wrmsr(ecx: u64, edx: u64, eax: u64) {
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") ecx,
            in("edx") edx,
            in("eax") eax,
            options(nostack, nomem, preserves_flags)
        );
    }
}

/// Write the APIC base address to
///
/// # Safety
///
/// Setting the APIC to an used, unmapped or otherwise invalid address can cause UB.
/// Presumably addresses larger than 4GiB are not allowed.
pub unsafe fn write_apic_base(addr: PhysicalAddress) {
    const APIC_ENABLE: u64 = 0x800;

    wrmsr(
        0x1B,
        (addr.as_u64() >> 32) & 0x0f,
        (addr.as_u64() & 0xfffff0000) | APIC_ENABLE,
    )
}
