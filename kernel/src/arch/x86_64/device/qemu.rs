use crate::arch::x86_64::port::*;
use crate::util::spin::SpinLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub struct Qemu {
    port: Port<u32, WriteOnly>,
}

impl Qemu {
    pub const unsafe fn new() -> Self {
        Self {
            port: Port::write_only(0xf4),
        }
    }

    pub fn exit(&mut self, code: ExitCode) -> ! {
        unsafe {
            self.port.write(code as u32);
        }

        loop {}
    }
}

pub static QEMU_DEVICE: SpinLock<Qemu> = SpinLock::new(unsafe { Qemu::new() });
