use core::ops::Deref;

use crate::util::spin::SpinOnce;

pub struct Singleton<T> {
    data: SpinOnce<T>,
    initializer: fn() -> T,
}

impl<T> Singleton<T> {
    pub const fn new(initializer: fn() -> T) -> Self {
        Self {
            data: SpinOnce::new(),
            initializer,
        }
    }
}

impl<T> Deref for Singleton<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.call_once(&self.initializer)
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test_case]
    fn test_deref_should_run_initializer() {
        let singleton = Singleton::new(|| true);
        assert!(*singleton);
    }

    #[test_case]
    fn test_singleton_should_only_initialize_once() {
        static INITIALIZE_COUNT: AtomicUsize = AtomicUsize::new(0);

        let singleton = Singleton::new(|| {
            INITIALIZE_COUNT.fetch_add(1, Ordering::AcqRel);
            true
        });

        assert!(*singleton);
        assert!(*singleton);
        assert!(*singleton);
        assert_eq!(INITIALIZE_COUNT.load(Ordering::Relaxed), 1);
    }
}
