use crate::RFlags;
use core::arch::asm;

/// (Atomically) enable interrupts.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

/// (Atomically) disable interrupts
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

/// Run a block of code (aka the `callback` argument) that is guaranteed to be executed without interrupts.
/// After completing the function, the interrupt status flag is restored to its original state.
#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub fn without_interrupts<F: FnOnce() -> R, R>(callback: F) -> R {
    let ints_enabled = RFlags::read().interrupts_enabled();

    if ints_enabled {
        disable_interrupts();
    }

    let ret = callback();

    if ints_enabled {
        enable_interrupts();
    }

    ret
}

#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub fn enable_interrupts_and_halt() {
    unsafe {
        asm!("sti; hlt", options(nomem, nostack));
    }
}
