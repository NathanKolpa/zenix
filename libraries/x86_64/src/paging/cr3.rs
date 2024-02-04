use core::arch::asm;
use essentials::address::PhysicalAddress;

const ADDR_MASK: u64 = 0x_000f_ffff_ffff_f000;

#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub fn active_page() -> PhysicalAddress {
    let value: u64;

    unsafe {
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    }

    PhysicalAddress::from(value & ADDR_MASK)
}

#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub unsafe fn set_active_page(page_addr: PhysicalAddress) {
    let value = page_addr.as_u64() | (!ADDR_MASK);
    // clears out the add, without removing the flags.
    asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub unsafe fn flush_page(page_addr: PhysicalAddress) {
    let addr = page_addr.as_usize();
    asm!("invlpg [{}]", in(reg) addr, options(nostack, preserves_flags));
}
