use essentials::spin::SpinLock;
use x86_64::device::qemu::Qemu;

pub static QEMU_DEVICE: SpinLock<Qemu> = SpinLock::new(unsafe { Qemu::new() });
