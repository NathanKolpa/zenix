use essentials::spin::SpinLock;
use x86_64::device::qemu::{ExitCode, Qemu};

use crate::utils::InterruptGuard;

static QEMU_DEVICE: InterruptGuard<SpinLock<Qemu>> =
    InterruptGuard::new_lock(unsafe { Qemu::new() });

pub fn shutdown_err() -> ! {
    QEMU_DEVICE.guard().lock().exit(ExitCode::Failed);
}

pub fn shutdown_ok() -> ! {
    QEMU_DEVICE.guard().lock().exit(ExitCode::Success);
}
