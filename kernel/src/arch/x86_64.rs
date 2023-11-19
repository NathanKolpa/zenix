//! All x86_64 specific stuff.

pub mod device;
pub mod interrupt;
pub mod port;
mod rflags;

pub use rflags::RFlags;
