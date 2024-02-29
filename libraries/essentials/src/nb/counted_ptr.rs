use core::sync::atomic::{AtomicPtr, Ordering};

pub struct CountedPtr<T> {
    ptr: AtomicPtr<T>,
}

impl<T> CountedPtr<T> {
    #[cfg(not(target_arch = "x86_64"))]
    const PTR_N_BITS: usize = core::mem::size_of::<usize>() * 8;

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

    #[cfg(target_arch = "x86_64")]
    fn apply_sign_ext(ptr: usize) -> usize {
        let extended = ptr >> (Self::PTR_N_BITS - 1);
        ptr | (Self::VALUE_MASK * extended)
    }

    pub fn new(ptr: *mut T, count: usize) -> Self {
        Self {
            ptr: AtomicPtr::new(Self::encode(ptr, count)),
        }
    }

    pub fn compare_exchange(
        &self,
        current: *mut T,
        new: *mut T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<*mut T, *mut T> {
        self.ptr.compare_exchange(current, new, success, failure)
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
        let count = 128;

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
}