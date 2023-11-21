//! All architecture specific code.

#[doc(cfg(target_arch = "x86_64"))]
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::init;

#[cfg(target_arch = "x86_64")]
pub use x86_64::NAME;
