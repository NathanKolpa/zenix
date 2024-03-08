pub mod pic_8259;
pub mod qemu;
pub mod uart_16550;

#[cfg(target_arch = "x86_64")]
#[doc(cfg(target_arch = "x86_64"))]
pub mod apic;

pub trait Serial {
    fn write_available(&self) -> bool;
    fn write_byte(&mut self, byte: u8);
}
