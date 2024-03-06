use core::ops::{Deref, DerefMut};

use essentials::nb::queue::QueueNode;

use crate::multitasking::scheduler::{thread::Thread, Scheduler};

pub struct ThreadBox {
    scheduler: &'static Scheduler,
    node: Option<&'static mut QueueNode<Thread>>,
}

impl Deref for ThreadBox {
    type Target = Thread;

    fn deref(&self) -> &Self::Target {
        self.node.as_ref().expect("Node should be set")
    }
}

impl DerefMut for ThreadBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node.as_mut().expect("Node should be set")
    }
}

impl Drop for ThreadBox {
    fn drop(&mut self) {
        let Some(node) = self.node.take() else {
            return;
        };

        self.scheduler.deallocate_thread(node);
    }
}

pub fn new_thread_box(
    scheduler: &'static Scheduler,
    node: Option<&'static mut QueueNode<Thread>>,
) -> ThreadBox {
    ThreadBox { scheduler, node }
}
