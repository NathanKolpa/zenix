use core::{
    cell::UnsafeCell,
    isize,
    marker::PhantomData,
    mem::{swap, MaybeUninit},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::array_uninit;

const CACHE_LINE_SIZE: usize = 64;
type CacheLinePadding = [u8; CACHE_LINE_SIZE];

/// A lock-free multi-producer multi-consumer bounded queue.
///
/// The algorithm is described by [Dmitry Vyukov](https://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue).
///
/// > The cost of enqueue/dequeue is 1 CAS per operation. No amortization, just 1 CAS. No dynamic memory allocation/management during operation. Producers and consumers are separated from each other (as in the two-lock queue), i.e. do not touch the same data while queue is not empty.
pub struct BoundedQueue<const SIZE: usize, T> {
    buffer: UnsafeCell<[MaybeUninit<T>; SIZE]>,
    sequence_buffer: [AtomicUsize; SIZE],
    _padding1: CacheLinePadding,
    enqueue_pos: AtomicUsize,
    _padding2: CacheLinePadding,
    dequeue_pos: AtomicUsize,
    _padding3: CacheLinePadding,
    _phantom: PhantomData<T>,
}

impl<const SIZE: usize, T> BoundedQueue<SIZE, T> {
    const BUFFER_MASK: usize = SIZE - 1;
    const SIZE_OK: () = assert!((SIZE >= 2) && ((SIZE & (SIZE - 1)) == 0));

    /// Construct a lock free queue.
    pub const fn new() -> Self {
        let _: () = Self::SIZE_OK;

        Self {
            buffer: UnsafeCell::new(array_uninit()),
            sequence_buffer: unsafe { core::mem::transmute_copy(&[0usize; SIZE]) },
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
            _padding1: [0; CACHE_LINE_SIZE],
            _padding2: [0; CACHE_LINE_SIZE],
            _padding3: [0; CACHE_LINE_SIZE],
            _phantom: PhantomData,
        }
    }

    pub fn spin_block_push(&self, mut value: T) {
        loop {
            match self.push(value) {
                Ok(_) => break,
                Err(v) => value = v,
            }
        }
    }

    /// Enqueue a value.
    ///
    /// # Returns
    ///
    /// A [`Result`] where:
    ///  - `Ok(())` If the operation is successful.
    ///  - The original value is returned wrapped in `Err`, if the queue is full or encounters an error during the operation.
    pub fn push(&self, value: T) -> Result<(), T> {
        let mut pos = self.enqueue_pos.load(Ordering::Relaxed);
        let mut unsafe_slot_ref;
        let mut sequence_ref;

        loop {
            let index = pos & Self::BUFFER_MASK;
            unsafe_slot_ref = unsafe { &mut (*self.buffer.get())[index] };
            sequence_ref = &self.sequence_buffer[index];

            let sequence = sequence_ref.load(Ordering::Acquire) + index;
            let diff = sequence as isize - pos as isize;

            match diff {
                d if d < 0 => return Err(value),
                0 => {
                    if self
                        .enqueue_pos
                        .compare_exchange_weak(pos, pos + 1, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                    {
                        break;
                    }
                }

                _ => {
                    pos = self.enqueue_pos.load(Ordering::Relaxed);
                }
            }
        }

        unsafe_slot_ref.write(value);
        sequence_ref.store(pos + 1, Ordering::Release);

        Ok(())
    }

    /// Dequeues a value
    ///
    /// # Returns
    ///
    /// An option with `Some` that contains the dequeued value. And `None` if the queue is empty.
    pub fn pop(&self) -> Option<T> {
        let mut pos = self.dequeue_pos.load(Ordering::Relaxed);
        let mut unsafe_slot_ref;
        let mut sequence_ref;

        loop {
            let index = pos & Self::BUFFER_MASK;
            unsafe_slot_ref = unsafe { &mut (*self.buffer.get()).as_mut()[index] };
            sequence_ref = &self.sequence_buffer[index];

            let sequence = sequence_ref.load(Ordering::Acquire);
            let diff = sequence as isize - (pos as isize + 1);

            match diff {
                0 => {
                    if self
                        .dequeue_pos
                        .compare_exchange_weak(pos, pos + 1, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                    {
                        break;
                    }
                }
                d if d < 0 => return None,
                _ => {
                    pos = self.dequeue_pos.load(Ordering::Relaxed);
                }
            }
        }

        let mut data = MaybeUninit::uninit();
        swap(&mut data, unsafe_slot_ref);

        sequence_ref.store(pos + Self::BUFFER_MASK + 1, Ordering::Release);

        Some(unsafe { data.assume_init() })
    }
}

unsafe impl<const SIZE: usize, T: Send + Sync> Sync for BoundedQueue<SIZE, T> {}
unsafe impl<const SIZE: usize, T: Send> Send for BoundedQueue<SIZE, T> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_push_pop() {
        let queue = BoundedQueue::<64, i32>::new();
        assert_eq!(Ok(()), queue.push(300));
        assert_eq!(Some(300), queue.pop());
    }

    #[test_case]
    fn test_pop_empty() {
        let queue = BoundedQueue::<64, i32>::new();
        assert_eq!(None, queue.pop());
        assert_eq!(None, queue.pop());
    }

    #[test_case]
    fn test_push_full() {
        let queue = BoundedQueue::<2, i32>::new();
        assert_eq!(Ok(()), queue.push(100));
        assert_eq!(Ok(()), queue.push(200));
        assert_eq!(Err(300), queue.push(300));
    }

    #[test_case]
    fn test_push_full_pop_empty() {
        let queue = BoundedQueue::<64, i32>::new();
        assert_eq!(Ok(()), queue.push(100));
        assert_eq!(Ok(()), queue.push(200));
        assert_eq!(Some(100), queue.pop());
        assert_eq!(Some(200), queue.pop());
        assert_eq!(None, queue.pop());
    }

    #[test_case]
    fn test_push_pop_lots() {
        let queue = BoundedQueue::<1024, i32>::new();

        for i in 0..1000 {
            assert_eq!(Ok(()), queue.push(i));
        }

        for i in 0..1000 {
            assert_eq!(Some(i), queue.pop());
        }
    }
}
