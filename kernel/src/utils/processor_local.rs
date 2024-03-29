use core::ops::Deref;

use alloc::{boxed::Box, vec::Vec};

use crate::arch::x86_64::mp::{processor_count, processor_id};

/// Like a thread local storage (TLS) but for processors, but unlike TLS this container does not
/// garantee thread safety.
pub struct ProcLocal<T> {
    proc_storage: Box<[T]>,
}

impl<T> ProcLocal<T> {
    pub fn new(mut factory: impl FnMut() -> T) -> Self {
        let count = processor_count();

        let mut vec = Vec::with_capacity(count);

        for _ in 0..count {
            vec.push(factory())
        }

        Self {
            proc_storage: vec.into_boxed_slice(),
        }
    }
}

impl<T> Deref for ProcLocal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.proc_storage[processor_id()]
    }
}
