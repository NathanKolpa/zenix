use core::{
    fmt::{Debug, Formatter},
    ops::Deref,
};

use essentials::spin::SpinLock;
use x86_64::{
    interrupt::{disable_interrupts, enable_interrupts},
    RFlags,
};

pub struct InterruptLockGuard<'a, T> {
    lock: &'a InterruptGuard<T>,
    ints_enabled: bool,
}

impl<T> Drop for InterruptLockGuard<'_, T> {
    fn drop(&mut self) {
        if self.ints_enabled {
            enable_interrupts()
        }
    }
}

impl<T> Deref for InterruptLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

pub struct InterruptGuard<T> {
    data: T,
}

impl<T> InterruptGuard<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }

    pub fn lock(&self) -> InterruptLockGuard<'_, T> {
        let ints_enabled = RFlags::read().interrupts_enabled();

        if ints_enabled {
            disable_interrupts();
        }

        InterruptLockGuard {
            lock: self,
            ints_enabled,
        }
    }
}

impl<T> InterruptGuard<SpinLock<T>> {
    pub const fn new_lock(data: T) -> Self {
        Self::new(SpinLock::new(data))
    }
}

impl<T> AsMut<T> for InterruptGuard<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Debug for InterruptGuard<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
