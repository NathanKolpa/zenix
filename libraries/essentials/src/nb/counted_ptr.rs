use core::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct CountedPtrVal<T> {
    ptr: *mut T,
    count: usize,
}

impl<T> Copy for CountedPtrVal<T> {}

impl<T> Clone for CountedPtrVal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for CountedPtrVal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr && self.count == other.count
    }
}

impl<T> CountedPtrVal<T> {
    pub fn next(self, new_ptr: *mut T) -> Self {
        Self {
            ptr: new_ptr,
            count: self.count + 1,
        }
    }

    pub fn inc(self) -> Self {
        self.next(self.mut_ptr())
    }

    pub fn ptr(&self) -> *const T {
        self.ptr
    }

    pub fn mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

/// A single (atomic) word containing a pointer + counter.
pub struct CountedPtr<T> {
    ptr: AtomicPtr<T>,
}

impl<T> CountedPtr<T> {
    #[cfg(not(target_arch = "x86_64"))]
    const PTR_N_BITS: usize = core::mem::size_of::<usize>() * 8 - 1;

    #[cfg(target_arch = "x86_64")]
    const PTR_N_BITS: usize = 48;

    const VALUE_MASK: usize = usize::MAX << Self::PTR_N_BITS;
    const PTR_MASK: usize = !Self::VALUE_MASK;

    fn encode(ptr: *mut T, count: usize) -> *mut T {
        let ptr_value = ptr as usize;
        let shifted_count = count << Self::PTR_N_BITS;

        ((ptr_value & Self::PTR_MASK) | shifted_count) as *mut T
    }

    fn decode(ptr: *mut T) -> (*mut T, usize) {
        let ptr_value = ptr as usize;
        let ptr = ptr_value & Self::PTR_MASK;

        #[cfg(target_arch = "x86_64")]
        let ptr = Self::apply_sign_ext(ptr);

        let count = (ptr_value & Self::VALUE_MASK) >> Self::PTR_N_BITS;

        (ptr as *mut T, count)
    }

    fn decode_val(ptr: *mut T) -> CountedPtrVal<T> {
        let (ptr, count) = Self::decode(ptr);
        CountedPtrVal { ptr, count }
    }

    #[cfg(target_arch = "x86_64")]
    fn apply_sign_ext(ptr: usize) -> usize {
        let extended = ptr >> (Self::PTR_N_BITS - 1);
        ptr | (Self::VALUE_MASK * extended)
    }

    pub const fn null() -> Self {
        Self {
            ptr: AtomicPtr::new(null_mut()),
        }
    }

    pub fn new(ptr: *mut T) -> Self {
        Self {
            ptr: AtomicPtr::new(Self::encode(ptr, 0)),
        }
    }

    pub fn load(&self, order: Ordering) -> CountedPtrVal<T> {
        Self::decode_val(self.ptr.load(order))
    }

    pub fn compare_exchange(
        &self,
        current: CountedPtrVal<T>,
        new: CountedPtrVal<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<CountedPtrVal<T>, CountedPtrVal<T>> {
        let encoded_current = Self::encode(current.ptr, current.count);
        let encoded_new = Self::encode(new.ptr, new.count);

        self.ptr
            .compare_exchange(encoded_current, encoded_new, success, failure)
            .map(Self::decode_val)
            .map_err(Self::decode_val)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_value_ptr_no_overlap() {
        let mut ptr_mask = CountedPtr::<()>::PTR_MASK;

        ptr_mask ^= CountedPtr::<()>::VALUE_MASK;
        ptr_mask ^= CountedPtr::<()>::VALUE_MASK;

        assert_eq!(CountedPtr::<()>::PTR_MASK, ptr_mask);
    }

    #[test_case]
    fn test_encode_decode() {
        let ptr = 0xdeadbeef as *mut ();
        let count = 127;

        let encoded = CountedPtr::encode(ptr, count);
        let (decoded_ptr, decoded_count) = CountedPtr::decode(encoded);

        assert_eq!(count, decoded_count);
        assert_eq!(ptr, decoded_ptr);
    }

    #[test_case]
    #[cfg(target_arch = "x86_64")]
    fn test_encode_decode_sing_ext_last_bit() {
        let ptr = (1usize << (CountedPtr::<()>::PTR_N_BITS - 1)) as *mut ();
        let count = 0;

        let encoded = CountedPtr::encode(ptr, count);
        let (decoded_ptr, decoded_count) = CountedPtr::decode(encoded);

        assert_eq!(count, decoded_count);
        assert_eq!(0xffff000000000000 | ptr as usize, decoded_ptr as usize);
    }

    #[test_case]
    fn test_encode_decode_sing_ext() {
        let ptr = usize::MAX as *mut ();
        let count = 0;

        let encoded = CountedPtr::encode(ptr, count);
        let (decoded_ptr, decoded_count) = CountedPtr::decode(encoded);

        assert_eq!(count, decoded_count);
        assert_eq!(ptr, decoded_ptr);
    }

    #[test_case]
    fn test_next() {
        let val = CountedPtrVal {
            ptr: 0xdeadbeef as *mut (),
            count: 0,
        };

        let new = val.next(0xbadb1d as *mut ());

        assert_ne!(val, new);
        assert_eq!(new.ptr(), 0xbadb1d as *const ());
        assert_eq!(new.count, 1);
    }

    #[test_case]
    fn test_inc() {
        let val = CountedPtrVal {
            ptr: 0xdeadbeef as *mut (),
            count: 0,
        };

        let new = val.inc();

        assert_ne!(val, new);
        assert_eq!(new.count, 1);
    }

    #[test_case]
    fn test_compare_exchange() {
        let ptr = 0xdeadbeef as *mut ();

        let counted = CountedPtr::new(ptr);

        let loaded = counted.load(Ordering::SeqCst);

        assert_eq!(ptr, loaded.mut_ptr());
        assert_eq!(0, loaded.count);

        let new = loaded.inc();
        assert_ne!(new, loaded);

        let new_result =
            counted.compare_exchange(loaded, loaded.inc(), Ordering::SeqCst, Ordering::Relaxed);

        assert!(new_result.is_ok());

        let failed = counted.compare_exchange(loaded, loaded, Ordering::SeqCst, Ordering::SeqCst);
        assert!(failed.is_err());
    }
}
