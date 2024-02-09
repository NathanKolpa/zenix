//! All x86_64 specific stuff.

#![cfg_attr(not(test), no_std)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner::runner)]
#![feature(abi_x86_interrupt)]
#![feature(doc_cfg)]

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub mod device;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
mod halt;

pub mod interrupt;
pub mod paging;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub mod port;

mod privilege;
mod rflags;
pub mod segmentation;
mod tables;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub use halt::*;
pub use privilege::PrivilegeLevel;
pub use rflags::RFlags;
pub use tables::DescriptorTablePointer;
