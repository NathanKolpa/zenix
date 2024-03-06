//! All x86_64 specific stuff.

mod ctx;
pub mod devices;
mod gdt;
mod idt;
mod init;
mod isr_wrapper;
pub use init::init;

pub const NAME: &str = "x86_64";

pub use ctx::CpuContext;
