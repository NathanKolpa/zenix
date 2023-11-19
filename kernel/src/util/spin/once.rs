use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

const INCOMPLETE_STATE: u8 = 0;
const LOCKED_STATE: u8 = 1;
const COMPLETE_STATE: u8 = 2;

/// The spin locking equivalent of `std::sync::Once`.
pub struct SpinOnce<T> {
    data: UnsafeCell<MaybeUninit<T>>,
    state: AtomicU8,
}

impl<T> SpinOnce<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(INCOMPLETE_STATE),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    unsafe fn data_ref(&self) -> &T {
        (*self.data.get()).assume_init_ref()
    }

    pub fn call_once(&self, initializer: impl FnOnce() -> T) -> &T {
        loop {
            match self.state.load(Ordering::Relaxed) {
                COMPLETE_STATE => return unsafe { self.data_ref() },
                LOCKED_STATE => {}
                INCOMPLETE_STATE => {
                    if self
                        .state
                        .compare_exchange(
                            INCOMPLETE_STATE,
                            LOCKED_STATE,
                            Ordering::Acquire,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        unsafe {
                            (*self.data.get()).write(initializer());
                        }

                        self.state.store(COMPLETE_STATE, Ordering::Release);
                        return unsafe { self.data_ref() };
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

unsafe impl<T: Send + Sync> Sync for SpinOnce<T> {}
unsafe impl<T: Send> Send for SpinOnce<T> {}
