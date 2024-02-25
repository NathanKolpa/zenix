use core::mem::MaybeUninit;

/// Create a new array of [`MaybeUninit`] elements without iterating over the array.
pub const fn array_uninit<const SIZE: usize, T>() -> [MaybeUninit<T>; SIZE] {
    unsafe { MaybeUninit::<[MaybeUninit<T>; SIZE]>::uninit().assume_init() }
}

/// Create a new array by constructing each element using a closure. Usefull for types that don't
/// implement [`Copy`].
pub fn array_init<const SIZE: usize, T>(mut construct: impl FnMut() -> T) -> [T; SIZE] {
    let mut array: MaybeUninit<[T; SIZE]> = MaybeUninit::uninit();

    let array_ptr = array.as_mut_ptr() as *mut T;

    for i in 0..SIZE {
        unsafe {
            let element_ptr = array_ptr.add(i);
            *element_ptr = construct();
        }
    }

    unsafe { array.assume_init() }
}
