use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::sync::atomic::{AtomicU8, Ordering};

const INCOMPLETE_STATE: u8 = 0;
const LOCKED_STATE: u8 = 1;
const COMPLETE_STATE: u8 = 2;

/// An uninitialized variable like [`MaybeUninit`] but safety is guaranteed because
/// when accessing (through [`Deref`]) uninitialized data, the thread will panic. This container can only be written to
/// a single time like [`crate::spin::SpinOnce`]. Writing more than once will result in a panic.
/// Hence the name, "PanicOnce".
/// This container is specifically designed for initializing static variables where `const fn` is not
/// possible.
pub struct PanicOnce<T> {
    state: AtomicU8,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for PanicOnce<T> {}
unsafe impl<T: Send> Send for PanicOnce<T> {}

impl<T> PanicOnce<T> {
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicU8::new(INCOMPLETE_STATE),
        }
    }

    pub fn initialize_with(&self, value: T) {
        if self
            .state
            .compare_exchange_weak(
                INCOMPLETE_STATE,
                LOCKED_STATE,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            panic!("PanicOnce is already initialized");
        }

        unsafe {
            (*self.data.get()).write(value);
        }

        self.state.store(COMPLETE_STATE, Ordering::Release);
    }

    fn guard(&self) {
        match self.state.load(Ordering::Relaxed) {
            COMPLETE_STATE => {}
            INCOMPLETE_STATE => panic!("PanicOnce is not initialized"),
            LOCKED_STATE => {
                panic!("PanicOnce is being initialized at the time of access by another thread")
            }
            _ => panic!("PanicOnce in an unexpected state"),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.state.load(Ordering::Relaxed) != COMPLETE_STATE
    }
}

impl<T> Deref for PanicOnce<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard();

        unsafe { (*self.data.get()).assume_init_ref() }
    }
}

impl<T> Drop for PanicOnce<T> {
    fn drop(&mut self) {
        if self.is_initialized() {
            unsafe { (*self.data.get()).assume_init_drop() }
        }
    }
}
