//! Spin lock type synchronization primitives.
//!
//! Spin locking means running a loop (spinning)
//! that does nothing while the primitive is locked until it is unlocked.
pub use lock::*;
pub use once::*;
pub use singleton::*;

mod lock;
mod once;
mod singleton;
