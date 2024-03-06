use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};

use crate::nb::CountedPtr;

pub struct QueueNode<T> {
    next: CountedPtr<QueueNode<T>>,
    value: MaybeUninit<T>,
}

impl<T> QueueNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            next: CountedPtr::null(),
            value: MaybeUninit::new(value),
        }
    }
}

impl<T> Deref for QueueNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.value.assume_init_ref() }
    }
}

impl<T> DerefMut for QueueNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.assume_init_mut() }
    }
}

pub struct DummyNode<T> {
    inner: QueueNode<T>,
}

impl<T> DummyNode<T> {
    pub const fn new() -> Self {
        Self {
            inner: QueueNode {
                next: CountedPtr::null(),
                value: MaybeUninit::uninit(),
            },
        }
    }
}

#[cfg(doc)]
use crate as essentials;

/// A *lock-free* multi-producer multi-consumer unbounded queue.
///
/// The algorithm is taken from the paper: [*Simple, Fast, and Practical Non-Blocking and Blocking Concurrent Queue Algorithms*](https://www.cs.rochester.edu/~scott/papers/1996_PODC_queues.pdf).
///
/// # On Allocation Safety
///
/// Within [Scott's Talk](https://www.youtube.com/watch?v=9XAx279s7gs) (one the authors of paper), problems are outlined on how this data structure interacts with dynamic memory management.
/// UB might occour when (for example):
/// 1. T1 reads head.
/// 2. T2 removes and free's dummy node, and then reuses it.
/// 3. T1 Tries to read dummy.next.
///
/// The proposed solution is to use something called a *type-preserving allocator*.
/// This special allocator ensures that allocated pointers always point to the same data type.
/// In contrast to typical allocators, where pointers could change the underlying data type upon reuse
/// after `free()`.
/// Despite the possiblity that a node is might already be freed, we have the garantee that what we think is a next
/// pointer, is indeed a next pointer.
///
/// The implementation of `Queue` does not rely on a type-preserving allocator.
/// Instead, it relies on the Rust typesystem.
/// The push, and pop operations deal with `&'static mut QueueNode`. The `'static` lifetime ensures that the
/// underlying memory of that type remains the same during the program's entire lifetime.
///
/// From an API perspective, the responsibility of managing nodes is then offloaded from the Queue itself to the consumer.
/// Making the usage of the Queue much more involved. But allowing the consumer to optimize allocation of nodes.
/// When looking at package/module graph, this API improves the design of the [`essentials`] crate. Since, the
/// dependecy of the [`alloc`] crate is not *strictly* required in order to use the Queue.
/// Which is prefered in a low-level crate.
///
/// A possible mechanism of managing nodes could be achieved by maintaining 2 Queues:
/// - Queue 1 contains all active nodes and can be used as normal.
/// - Queue 2 contains all "retired" nodes.
/// - Call [`alloc::boxed::Box::leak`] for new allocated nodes, but only when Queue 2 is empty.
/// - (Optionally) A atomic counter that makes sure that the amount of leaked memory is limited.
/// - When a node is 'deallocated', it gets pushed to Queue 2.
pub struct Queue<T> {
    head: CountedPtr<QueueNode<T>>,
    tail: CountedPtr<QueueNode<T>>,
}

impl<T> Queue<T> {
    pub fn new(dummy: &'static mut DummyNode<T>) -> Self {
        // Original: Allocate a free node
        // Changed: Get the refrence to the free node
        let dummy_ptr = (&mut dummy.inner) as *mut QueueNode<T>;

        // Original: Make it the only node in the linked list
        // Changed: The constructor of DummyNode ensures that next is null

        // Both Head and Tail point to it
        Self {
            head: CountedPtr::new(dummy_ptr),
            tail: CountedPtr::new(dummy_ptr),
        }
    }

    pub fn push(&self, new_node: &'static mut QueueNode<T>) {
        // Set next pointer of node to NULL
        new_node.next = CountedPtr::null();

        let mut tail;

        // Keep trying until Enqueue is done
        loop {
            // Read Tail.ptr and Tail.count together
            tail = self.tail.load(Ordering::SeqCst);
            let tail_node = unsafe { &*(tail.ptr()) };

            // Read next ptr and count fields together
            let next = tail_node.next.load(Ordering::SeqCst);

            //  Are tail and next consistent?
            if tail == self.tail.load(Ordering::SeqCst) {
                // Was Tail pointing to the last node?
                if next.ptr().is_null() {
                    // Try to link node at the end of the linked list.
                    if tail_node
                        .next
                        .compare_exchange(
                            next,
                            next.next(new_node),
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        // Enqueue is done. Exit loop
                        break;
                    }
                } else {
                    // Tail was not pointing to the last node
                    // Try to swing Tail to the next node
                    let _ = self.tail.compare_exchange(
                        tail,
                        tail.next(next.mut_ptr()),
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    );
                }
            }
        }

        // Enqueue is done. Try to swing Tail to the inserted node
        let _ = self.tail.compare_exchange(
            tail,
            tail.next(new_node),
            Ordering::SeqCst,
            Ordering::Relaxed,
        );
    }

    pub fn pop(&self) -> Option<&'static mut QueueNode<T>> {
        // Keep trying until Dequeue is done
        loop {
            //  Read Head
            let head = self.head.load(Ordering::SeqCst);

            //  Read Tail
            let tail = self.tail.load(Ordering::SeqCst);

            // Read Head.ptr–>next
            let head_node = unsafe { &mut *(head.mut_ptr()) };
            let next = head_node.next.load(Ordering::SeqCst);

            // Are head, tail, and next consistent?
            if head == self.head.load(Ordering::SeqCst) {
                // Is queue empty or Tail falling behind?
                if head.ptr() == tail.ptr() {
                    // Is queue empty?
                    if next.ptr().is_null() {
                        // Queue is empty, couldn’t dequeue
                        return None;
                    }

                    // Tail is falling behind. Try to advance it
                    let _ = self.tail.compare_exchange(
                        tail,
                        tail.next(next.mut_ptr()),
                        Ordering::SeqCst,
                        Ordering::Relaxed,
                    );
                } else {
                    // No need to deal with Tail

                    // Original: Read value before CAS, otherwise another dequeue might free the next node.
                    // Changed: The rust typesystem guarantees that the next node lives longer that
                    // the queue.

                    let next_val = unsafe { &mut *next.mut_ptr() };

                    // Try to swing Head to the next node
                    if self
                        .head
                        .compare_exchange(
                            head,
                            head.next(next.mut_ptr()),
                            Ordering::SeqCst,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        core::mem::swap(&mut next_val.value, &mut head_node.value);

                        // Dequeue is done. Exit loop
                        return Some(head_node);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_pop_empty() {
        let dummy = Box::leak(Box::new(DummyNode::new()));
        let queue = Queue::<u8>::new(dummy);

        assert_eq!(None, queue.pop().map(|x| **x));
    }

    #[test_case]
    fn test_push_pop() {
        let dummy = Box::leak(Box::new(DummyNode::new()));
        let node_1 = Box::leak(Box::new(QueueNode::new(123)));

        let queue = Queue::new(dummy);
        queue.push(node_1);

        let pop = queue.pop();
        assert_eq!(Some(123), pop.map(|x| **x));

        assert_eq!(None, queue.pop().map(|x| **x));
    }

    #[test_case]
    fn test_push_pop_twice() {
        let dummy = Box::leak(Box::new(DummyNode::new()));
        let node_1 = Box::leak(Box::new(QueueNode::new(123)));
        let node_2 = Box::leak(Box::new(QueueNode::new(321)));

        let queue = Queue::new(dummy);
        queue.push(node_1);
        queue.push(node_2);

        let pop = queue.pop();
        assert_eq!(Some(123), pop.map(|x| **x));

        let pop = queue.pop();
        assert_eq!(Some(321), pop.map(|x| **x));

        assert_eq!(None, queue.pop().map(|x| **x));
    }

    #[test_case]
    fn test_push_pop_reuse() {
        let dummy = Box::leak(Box::new(DummyNode::new()));
        let node_1 = Box::leak(Box::new(QueueNode::new(123)));

        let queue = Queue::new(dummy);

        queue.push(node_1);
        let node_1 = queue.pop().unwrap();
        assert_eq!(123, **node_1);

        queue.push(node_1);
        assert_eq!(123, **queue.pop().unwrap());
    }
}
