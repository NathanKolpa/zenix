use crate::port::*;
use essentials::spin::SpinLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success = 19,
    Failed = 33,
}
pub struct Qemu {
    exit: Port<u8, WriteOnly>,
}

impl Qemu {
    pub const unsafe fn new() -> Self {
        Self {
            exit: Port::write_only(0x501),
        }
    }

    pub fn exit(&mut self, code: ExitCode) -> ! {
        unsafe {
            self.exit.write((code as u8) >> 1 | 1);
        }

        panic!("Processor still running after requesting QEMU exit. Are you rinning in a qemu emulator? Or have you forgot to  include the '-device isa-debug-exit' flag?");
    }
}

pub static QEMU_DEVICE: SpinLock<Qemu> = SpinLock::new(unsafe { Qemu::new() });
