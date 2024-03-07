use essentials::spin::SpinLock;
use x86_64::device::qemu::Qemu;

use crate::utils::InterruptGuard;

pub static QEMU_DEVICE: InterruptGuard<SpinLock<Qemu>> =
    InterruptGuard::new_lock(unsafe { Qemu::new() });
