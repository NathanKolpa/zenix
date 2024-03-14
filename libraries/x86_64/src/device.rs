mod pic_8259;
mod pit;
pub mod qemu;
mod uart_16550;

mod apic;

pub use apic::*;
pub use pic_8259::*;
pub use pit::*;
pub use uart_16550::*;

pub trait Serial {
    fn write_available(&self) -> bool;
    fn write_byte(&mut self, byte: u8);
}
