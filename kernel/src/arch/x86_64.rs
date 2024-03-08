//! All x86_64 specific stuff.

mod ctx;
pub mod devices;
mod gdt;
mod idt;
mod info;
mod init;
mod isr_wrapper;
mod int_control;
pub use info::print_info;
pub use init::init;

pub const NAME: &str = "x86_64";

pub use ctx::CpuContext;
