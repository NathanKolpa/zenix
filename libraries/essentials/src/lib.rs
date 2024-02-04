//! Simple containers and primitives.

#![feature(doc_cfg)]

pub mod address;
mod bitmap;
pub mod display;
mod fixed_vec;
mod panic_once;
pub mod spin;
mod static_ptr;

pub use bitmap::Bitmap;
pub use fixed_vec::FixedVec;
pub use panic_once::PanicOnce;
pub use static_ptr::StaticPtr;
