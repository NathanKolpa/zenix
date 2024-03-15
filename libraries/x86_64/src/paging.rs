pub use entry::*;
pub use flags::*;
pub use page_table::*;
pub use size::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub mod cr2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub mod cr3;
mod entry;
mod flags;
mod page_table;
mod size;
