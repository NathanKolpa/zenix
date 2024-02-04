//! All x86_64 specific stuff.

pub mod devices;
mod gdt;
mod idt;
mod init;
pub use init::init;

pub const NAME: &str = "x86_64";
