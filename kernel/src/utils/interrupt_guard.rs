use core::{
    fmt::{Debug, Formatter},
    ops::Deref,
};

use essentials::spin::{SpinLock, SpinLockGuard};
use x86_64::{
    interrupt::{disable_interrupts, enable_interrupts},
    RFlags,
};

/// A container that ensures interrupts are disabled when the inner data is accessed and restores
/// the interrupt enable flag when the data is no longer accessed.
/// Safety should not rely on the fact that interrupts are actually disabled. It's still perfectly
/// safe to call [`enable_interrupts`] while a [`InterruptLockGuard`] is active.
/// The use-case for this container is to prevent deadlocks when using [`SpinLock`], so that the
/// user cannot forget to disable interrupts.
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

    pub fn guard(&self) -> InterruptLockGuard<'_, T> {
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

pub struct InterruptSpinLockGuard<'a, T> {
    lock: SpinLockGuard<'a, T>,
    guard: InterruptLockGuard<'a, SpinLock<T>>,
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
