use essentials::spin::{Singleton, SpinLock};
use x86_64::device::{qemu::Qemu, uart_16550::Uart16550};

pub static UART_16550_CHANNEL: Singleton<SpinLock<Uart16550>> =
    Singleton::new(|| SpinLock::new(unsafe { Uart16550::new_and_init(0x3F8) }));

pub static QEMU_DEVICE: SpinLock<Qemu> = SpinLock::new(unsafe { Qemu::new() });
