use core::arch::asm;
use essentials::address::VirtualAddress;

#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub fn page_fault_addr() -> VirtualAddress {
    let value: u64;

    unsafe {
        asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
    }

    value.into()
}
