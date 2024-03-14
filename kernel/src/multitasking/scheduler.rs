mod error;
mod thread;
mod thread_box;

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::{arch::CpuContext, utils::ProcLocal};

use alloc::boxed::Box;
use essentials::{
    nb::{
        queue::{DummyNode, QueueNode},
        Queue,
    },
    spin::SpinLock,
    FixedVec, PanicOnce,
};

pub use thread::*;
use thread_box::new_thread_box;

pub use error::SchedulerError;
pub use thread_box::ThreadBox;

const MAX_THREADS: usize = 1024 * 10;
const PRIORITY_LEVELS: usize = 1;

// TOOD: nodes and dummy nodes to the eternal alloc.

pub struct Scheduler {
    id_autoincrement: AtomicUsize,
    run_queues: PanicOnce<FixedVec<PRIORITY_LEVELS, Queue<Thread>>>,
    retired_threads: PanicOnce<Queue<Thread>>,
    allocated_threads: AtomicUsize,
    allocation_exceeded: AtomicBool,

    current_thread: PanicOnce<ProcLocal<SpinLock<Option<&'static mut QueueNode<Thread>>>>>,
    current_thread_id: PanicOnce<ProcLocal<AtomicUsize>>,
}

impl Scheduler {
    const fn new() -> Self {
        Self {
            id_autoincrement: AtomicUsize::new(0),
            run_queues: PanicOnce::new(),
            retired_threads: PanicOnce::new(),
            allocated_threads: AtomicUsize::new(0),
            allocation_exceeded: AtomicBool::new(false),

            current_thread: PanicOnce::new(),
            current_thread_id: PanicOnce::new(),
        }
    }

    /// Initialize the scheduler.
    ///
    /// # Panics/Allocation
    ///
    /// This function allocates memory in the heap of a pre determined size, and never
    /// de-allocates any memory. If memory cannot be allocated, the thread will panic.
    pub fn init(&self) {
        let mut queues = FixedVec::new();

        for _ in 0..PRIORITY_LEVELS {
            let dummy = Box::leak(Box::new(DummyNode::new()));
            queues.push(Queue::new(dummy));
        }

        self.run_queues.initialize_with(queues);

        self.retired_threads
            .initialize_with(Queue::new(Box::leak(Box::new(DummyNode::new()))));

        self.current_thread
            .initialize_with(ProcLocal::new(|| SpinLock::new(None)));

        self.current_thread_id
            .initialize_with(ProcLocal::new(|| AtomicUsize::new(0)));
    }

    pub fn current_as_thread(&self, priority: ThreadPriority) -> Result<ThreadId, SchedulerError> {
        let tid = self.alloc_thread_id();

        let mut current_thread_lock = self.current_thread.lock();

        if self
            .current_thread_id
            .compare_exchange(0, tid, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(SchedulerError::SlotTaken);
        }

        let node = self.spawn_id(tid, priority, Default::default())?;

        *current_thread_lock = Some(node);

        Ok(tid)
    }

    pub fn next_ctx(&self, current: CpuContext) -> Option<CpuContext> {
        let mut current_node_lock = self.current_thread.lock();

        if let Some(current_node) = current_node_lock.take() {
            current_node.save_context(current);
            self.schedule_node(current_node);
        }

        let next_node = self.next_node()?;
        let ctx = next_node.context();

        *current_node_lock = Some(next_node);

        Some(ctx)
    }

    pub fn spawn_thread(
        &self,
        priority: ThreadPriority,
        context: CpuContext,
    ) -> Result<ThreadId, SchedulerError> {
        let new_thread_id = self.alloc_thread_id();

        let node = self.spawn_id(new_thread_id, priority, context)?;
        self.schedule_node(node);

        Ok(new_thread_id)
    }

    fn alloc_thread_id(&self) -> ThreadId {
        self.id_autoincrement.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn spawn_id(
        &self,
        new_thread_id: ThreadId,
        priority: ThreadPriority,
        context: CpuContext,
    ) -> Result<&'static mut QueueNode<Thread>, SchedulerError> {
        self.allocate_thread(new_thread_id, self.current_thread_id(), priority, context)
    }

    fn next_node(&self) -> Option<&'static mut QueueNode<Thread>> {
        self.run_queues.iter().find_map(|q| q.pop())
    }

    fn schedule_node(&self, thread_node: &'static mut QueueNode<Thread>) {
        let index = thread_node.priority_index::<PRIORITY_LEVELS>();

        let queue = self
            .run_queues
            .get(index)
            .or_else(|| self.run_queues.last())
            .expect("there should at least be one run queue");

        queue.push(thread_node);
    }

    fn deallocate_thread(&self, thread_node: &'static mut QueueNode<Thread>) {
        self.retired_threads.push(thread_node);
    }

    fn box_in(&'static self, thread_node: &'static mut QueueNode<Thread>) -> ThreadBox {
        new_thread_box(self, Some(thread_node))
    }

    fn allocate_thread(
        &self,
        thread: ThreadId,
        spawned_by: Option<ThreadId>,
        priority: ThreadPriority,
        context: CpuContext,
    ) -> Result<&'static mut QueueNode<Thread>, SchedulerError> {
        let new_thread = Thread::new(thread, spawned_by, priority, context);

        if let Some(retired) = self.retired_threads.pop() {
            **retired = new_thread;
            return Ok(retired);
        }

        if self.allocation_exceeded.load(Ordering::Relaxed) {
            return Err(SchedulerError::ThreadLimit);
        }

        let old_count = self.allocated_threads.fetch_add(1, Ordering::Relaxed);

        if old_count + 1 > MAX_THREADS {
            self.allocation_exceeded.store(true, Ordering::Relaxed);
            return Err(SchedulerError::ThreadLimit);
        }

        let new_node_alloc =
            Box::try_new(QueueNode::new(new_thread)).map_err(|_| SchedulerError::OutOfMemory)?;

        Ok(Box::leak(new_node_alloc))
    }

    fn current_thread_id(&self) -> Option<ThreadId> {
        let val = self.current_thread_id.load(Ordering::Relaxed);

        if val == 0 {
            return None;
        }

        Some(val)
    }
}

pub static SCHEDULER: Scheduler = Scheduler::new();
