use core::{
    cell::UnsafeCell,
    isize,
    marker::PhantomData,
    mem::{swap, MaybeUninit},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::array_init;

const CACHE_LINE_SIZE: usize = 64;
type CacheLinePadding = [u8; CACHE_LINE_SIZE];

pub struct Slot<T> {
    sequence: AtomicUsize,
    data: MaybeUninit<T>,
}

impl<T> Slot<T> {
    fn new_uninit() -> Self {
        Self {
            sequence: AtomicUsize::new(0),
            data: MaybeUninit::uninit(),
        }
    }
}

/// A lock-free multi-producer multi-consumer bounded queue.
///
/// The algorithm is described by [Dmitry Vyukov](https://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue).
///
/// > The cost of enqueue/dequeue is 1 CAS per operation. No amortization, just 1 CAS. No dynamic memory allocation/management during operation. Producers and consumers are separated from each other (as in the two-lock queue), i.e. do not touch the same data while queue is not empty.
pub struct BoundedQueue<S, T> {
    buffer: UnsafeCell<S>,
    buffer_mask: usize,
    _padding1: CacheLinePadding,
    enqueue_pos: AtomicUsize,
    _padding2: CacheLinePadding,
    dequeue_pos: AtomicUsize,
    _padding3: CacheLinePadding,
    _phantom: PhantomData<T>,
}

impl<S, T> BoundedQueue<S, T>
where
    S: AsMut<[Slot<T>]>,
{
    fn new_and_init(mut buffer: S) -> Self {
        Self {
            buffer_mask: buffer.as_mut().len() - 1,
            buffer: UnsafeCell::new(buffer),
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
            _padding1: [0; CACHE_LINE_SIZE],
            _padding2: [0; CACHE_LINE_SIZE],
            _padding3: [0; CACHE_LINE_SIZE],
            _phantom: PhantomData,
        }
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let mut pos = self.enqueue_pos.load(Ordering::Relaxed);
        let mut unsafe_cell_ref;

        loop {
            let index = pos & self.buffer_mask;
            unsafe_cell_ref = unsafe { &mut (*self.buffer.get()).as_mut()[index] };

            let sequence = unsafe_cell_ref.sequence.load(Ordering::Acquire);
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

        unsafe_cell_ref.data.write(value);
        unsafe_cell_ref.sequence.store(pos + 1, Ordering::Release);

        Ok(())
    }

    pub fn pop(&self) -> Option<T> {
        let mut pos = self.dequeue_pos.load(Ordering::Relaxed);
        let mut unsafe_cell_ref;

        loop {
            let index = pos & self.buffer_mask;
            unsafe_cell_ref = unsafe { &mut (*self.buffer.get()).as_mut()[index] };

            let sequence = unsafe_cell_ref.sequence.load(Ordering::Acquire);
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
        swap(&mut data, &mut unsafe_cell_ref.data);

        unsafe_cell_ref.sequence.store(pos + self.buffer_mask + 1, Ordering::Release);

        Some(unsafe { data.assume_init() })
    }
}

impl<T, const SIZE: usize> BoundedQueue<[Slot<T>; SIZE], T> {
    const SIZE_OK: () = assert!((SIZE >= 2) && ((SIZE & (SIZE - 1)) == 0));

    // TODO: make this function const
    pub fn new_inline() -> Self {
        let _: () =  Self::SIZE_OK ;
        Self::new_and_init(array_init(|| Slot::new_uninit()))
    }
}

/// A [`BoundQueue`] with a array as storage.
pub type ArrayBoundQueue<const SIZE: usize, T> = BoundedQueue<[Slot<T>; SIZE], T>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_push_pop() {
        let queue = ArrayBoundQueue::<64, _>::new_inline();
        assert_eq!(Ok(()), queue.push(300));
        assert_eq!(Some(300), queue.pop());
    }

    #[test_case]
    fn test_pop_empty() {
        let queue = ArrayBoundQueue::<64, i32>::new_inline();
        assert_eq!(None, queue.pop());
        assert_eq!(None, queue.pop());
    }

    #[test_case]
    fn test_push_full() {
        let queue = ArrayBoundQueue::<2, i32>::new_inline();
        assert_eq!(Ok(()), queue.push(100));
        assert_eq!(Err(200), queue.push(200));
    }

    #[test_case]
    fn test_push_full_pop_empty() {
        let queue = ArrayBoundQueue::<2, i32>::new_inline();
        assert_eq!(Ok(()), queue.push(100));
        assert_eq!(Err(200), queue.push(200));
        assert_eq!(Some(100), queue.pop());
        assert_eq!(None, queue.pop());
    }
}


