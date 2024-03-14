//! All x86_64 specific stuff.

mod acpi;
mod gdt;
mod init;
mod interrupts;
pub mod mp;
pub mod shutdown;
pub use init::init;

pub const NAME: &str = "x86_64";

pub use interrupts::CpuContext;
