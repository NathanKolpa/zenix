use core::cell::UnsafeCell;
use core::fmt::{Debug, Formatter};
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

/// The spin locking equivalent of `std::sync::Mutex`.
pub struct SpinLock<T> {
    data: UnsafeCell<T>,
    is_locked: AtomicBool,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            is_locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .is_locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }

        SpinLockGuard { lock: self }
    }

    fn unlock(&self) {
        self.is_locked.store(false, Ordering::Release);
    }
}

impl<T> AsMut<T> for SpinLock<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

unsafe impl<T: Send + Sync> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> Debug for SpinLock<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let lock = self.lock();
        write!(f, "{:?}", *lock)
    }
}
