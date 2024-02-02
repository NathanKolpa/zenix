use core::sync::atomic::{AtomicPtr, Ordering};

pub struct StaticPtr<T> {
    ptr: AtomicPtr<T>,
}

impl<T> StaticPtr<T> {
    pub const unsafe fn new(ptr: *mut T) -> Self {
        Self {
            ptr: AtomicPtr::new(ptr),
        }
    }
}

impl<T> StaticPtr<T>
where
    T: Send,
{
    pub fn take(&self) -> Option<&'static mut T> {
        let ptr = self.ptr.swap(core::ptr::null_mut(), Ordering::SeqCst);

        if ptr.is_null() {
            return None;
        }

        Some(unsafe { &mut *ptr })
    }
}
