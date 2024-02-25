//! Simple containers and primitives.

#![cfg_attr(not(test), no_std)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner::runner)]
#![feature(doc_cfg)]

pub mod address;
mod bitmap;
pub mod display;
mod fixed_vec;
mod panic_once;
pub mod spin;
mod static_ptr;
pub mod nb;
mod array_init;

pub use bitmap::Bitmap;
pub use fixed_vec::FixedVec;
pub use panic_once::PanicOnce;
pub use static_ptr::StaticPtr;
pub use array_init::*;
