use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::{ptr, slice};

use crate::array_uninit;

/// A vector with a capacity known at compile-time.
/// Because the fixed size, the vector can be stored entirely inline and on the stack.
/// No heap allocator is thus required.
pub struct FixedVec<const SIZE: usize, T> {
    len: usize,
    elements: [MaybeUninit<T>; SIZE],
}

impl<const SIZE: usize, T> FixedVec<SIZE, T> {
    pub const fn new() -> Self {
        FixedVec {
            len: 0,
            elements: array_uninit::<SIZE, T>(),
        }
    }

    pub fn initialized_with(value: T) -> Self
    where
        T: Clone,
    {
        let mut vec = Self::new();

        for _ in 0..SIZE {
            vec.push(value.clone());
        }

        vec
    }

    pub fn is_full(&self) -> bool {
        self.len == SIZE
    }

    pub fn push(&mut self, value: T) {
        // Bounds-checking is done by the compiler.
        self.elements[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        match self.len {
            0 => None,
            _ => {
                self.len -= 1;
                unsafe { Some(ptr::read(self.as_ptr().add(self.len))) }
            }
        }
    }

    pub const fn as_ptr(&self) -> *const T {
        self.elements.as_ptr() as *const T
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.elements.as_ptr() as *const T, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.elements.as_ptr() as *mut T, self.len) }
    }

    pub fn clear(&mut self) {
        self.shrink_to(0);
    }

    pub fn extend_to(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        assert!(new_len > self.len);
        assert!(new_len < SIZE);

        for item in &mut self.elements[self.len..new_len] {
            item.write(value.clone());
        }

        self.len = new_len;
    }

    pub fn shrink_to(&mut self, new_len: usize) {
        assert!(new_len < self.len);

        for item in &mut self.elements[new_len..self.len] {
            unsafe { item.assume_init_drop() }
        }

        self.len = new_len;
    }
}

impl<T, const SIZE: usize> Deref for FixedVec<SIZE, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const SIZE: usize> DerefMut for FixedVec<SIZE, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const SIZE: usize> Drop for FixedVec<SIZE, T> {
    fn drop(&mut self) {
        for item in &mut self.elements[0..self.len] {
            unsafe { item.assume_init_drop() }
        }
    }
}

impl<T, const SIZE: usize> Index<usize> for FixedVec<SIZE, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len);
        unsafe { self.elements[index].assume_init_ref() }
    }
}

impl<T, const SIZE: usize> IndexMut<usize> for FixedVec<SIZE, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);
        unsafe { self.elements[index].assume_init_mut() }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;

    use super::*;

    struct DropTester<'a> {
        count: &'a RefCell<i32>,
    }

    impl<'a> DropTester<'a> {
        pub fn new(count: &'a RefCell<i32>) -> Self {
            *count.borrow_mut() += 1;

            Self { count }
        }
    }

    impl<'a> Drop for DropTester<'a> {
        fn drop(&mut self) {
            *self.count.borrow_mut() -= 1;
        }
    }

    #[test_case]
    fn test_push_pop() {
        let mut vec = FixedVec::<100, _>::new();
        assert_eq!(0, vec.len());

        vec.push(2001);
        assert_eq!(1, vec.len());

        let popped = vec.pop().unwrap();
        assert_eq!(0, vec.len());
        assert_eq!(2001, popped);
    }

    #[test_case]
    fn test_as_slice() {
        let mut vec = FixedVec::<100, _>::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        let slice = vec.as_slice();
        assert_eq!([10, 20, 30], slice);
    }

    #[test_case]
    fn test_modify_mut_slice() {
        let mut vec = FixedVec::<100, _>::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        let slice = vec.as_mut_slice();
        slice[1] = 60;

        assert_eq!(60, vec[1]);
    }

    #[test_case]
    fn test_drop_elements() {
        let instance_count = RefCell::new(0);

        let mut vec = FixedVec::<100, _>::new();
        vec.push(DropTester::new(&instance_count));
        vec.push(DropTester::new(&instance_count));

        assert_eq!(*instance_count.borrow(), 2);

        drop(vec);

        assert_eq!(*instance_count.borrow(), 0);
    }

    #[test_case]
    fn test_shrink_drop_elements() {
        let instance_count = RefCell::new(0);

        let mut vec = FixedVec::<100, _>::new();
        vec.push(DropTester::new(&instance_count));
        vec.push(DropTester::new(&instance_count));
        vec.push(DropTester::new(&instance_count));
        vec.push(DropTester::new(&instance_count));

        assert_eq!(*instance_count.borrow(), 4);

        vec.shrink_to(2);

        assert_eq!(*instance_count.borrow(), 2);
    }

    #[test_case]
    fn test_extend_to() {
        let mut vec = FixedVec::<100, _>::new();
        vec.push(10);
        vec.push(20);

        vec.extend_to(5, 30);
        assert_eq!(5, vec.len());
        assert_eq!([10, 20, 30, 30, 30], vec.as_slice());
    }
}
