//! All x86_64 specific stuff.

pub mod device;
mod halt;
pub mod init;
pub mod interrupt;
pub mod port;
mod privilege;
mod rflags;
pub mod segmentation;
mod tables;

pub use halt::*;
pub use privilege::PrivilegeLevel;
pub use rflags::RFlags;
pub use tables::DescriptorTablePointer;

pub const NAME: &'static str = "x86_64";
