use core::{arch::asm, u64};

use essentials::address::PhysicalAddress;

/// RDMSR — Read From Model Specific Register
///
/// From [felixcloutier](https://www.felixcloutier.com/x86/rdmsr):
///
/// > Reads the contents of a 64-bit model specific register (MSR) specified in the ECX register into registers EDX:EAX.
/// > (On processors that support the Intel 64 architecture, the high-order 32 bits of RCX are ignored.)
/// > The EDX register is loaded with the high-order 32 bits of the MSR and the EAX register is loaded with the low-order 32 bits.
/// > (On processors that support the Intel 64 architecture, the high-order 32 bits of each of RAX and RDX are cleared.)
/// > If fewer than 64 bits are implemented in the MSR being read, the values returned to EDX:EAX in unimplemented bit locations are undefined.
///
/// # Safety
///
/// The value passed in `ecx` must be valid.
pub unsafe fn rdmsr(ecx: u64) -> (u64, u64) {
    let edx: u64;
    let eax: u64;

    unsafe {
        asm!(
            "rdmsr",
            in("ecx") ecx,
            out("edx") edx,
            out("eax") eax,
            options(nostack, nomem, preserves_flags)
        );
    }

    (edx, eax)
}

/// Get the physical address of the APIC registers page.
pub fn read_apic_base() -> PhysicalAddress {
    let (edx, eax) = unsafe { rdmsr(0x1B) };
    PhysicalAddress::from((eax & 0xfffff000) | ((edx & 0x0f) << 32))
}
