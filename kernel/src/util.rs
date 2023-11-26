//! Simple containers and primitives.

pub mod address;
pub mod display;
mod fixed_vec;
mod panic_once;
pub mod spin;

pub use fixed_vec::FixedVec;
pub use panic_once::PanicOnce;
