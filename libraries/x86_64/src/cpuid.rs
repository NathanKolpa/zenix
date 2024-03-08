mod cpu_features;

pub use cpu_features::CpuFeatures;

use core::{arch::asm, u64};

/// Call the cpuid instruction
///
/// # Safety
///
/// Passing eax with an invalid value can cause UB.
pub unsafe fn cpuid(mut eax: u64) -> (u64, u64, u64, u64) {
    let ecx;
    let edx;
    let ebx;

    asm!(
        "push rbx",
        "cpuid",
        "mov rdi, rbx",
        "pop rbx",
        inout("eax") eax,
        out("ecx") ecx,
        out("edx") edx,
        out("rdi") ebx,
        options(nomem, preserves_flags)
    );

    (eax, ebx, ecx, edx)
}
pub fn read_features() -> CpuFeatures {
    let (_eax, _ebx, ecx, edx) = unsafe { cpuid(1) };

    CpuFeatures::new(ecx, edx)
}
