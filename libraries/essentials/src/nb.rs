//! Nonblocking data structures.
//!
//! The progress of operations within nonblocking data structure are categorised in one of three classes ([Michael Scott](https://www.youtube.com/watch?v=9XAx279s7gs&t=3390s)). Each category is a guarantee of how long an operation might take based on what other threads are doing. The classes are listed based on how strong their guarantees are:
//!
//! 1. **Wait-free**: The strongest class. The number of steps within an operation is not
//!    influenced by  what other threads are doing. My thread never starves.
//! 2. **Lock-free**: The intermediate class. An operation in at least one thread is gauranteed to complete in a finite number of steps. The system never starves.
//! 3. **Obstruction-free**: The weakest class. An operation is guaranteed to complete only if there
//!    are no other threads interfering. System starvation might occur but can be resolved by
//!    external mechanisms.

pub mod bounded_queue;

pub use bounded_queue::ArrayBoundQueue;
