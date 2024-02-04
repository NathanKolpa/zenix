pub use context::*;
pub use gate::*;
pub use idt::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
pub use instructions::*;

mod context;
mod gate;
mod idt;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[doc(cfg(any(target_arch = "x86_64", target_arch = "x86")))]
mod instructions;
