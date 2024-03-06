use core::{arch::asm, u64};

pub unsafe fn rdmsr(eax: u64) -> (u64, u64) {
    let edx: u64;
    let ecx: u64;

    unsafe {
        asm!(
            "rdmsr",
            in("eax") eax,
            out("edx") edx,
            out("ecx") ecx,
            options(nostack, nomem, preserves_flags)
        );
    }

    (edx, ecx)
}
