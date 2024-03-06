use crate::arch::CpuContext;

pub type ThreadId = usize;

pub type ThreadPriority = u8;

pub const LOWEST_PRIORITY: ThreadPriority = ThreadPriority::MIN;

pub struct Thread {
    thread_id: ThreadId,
    spawned_by: Option<ThreadId>,
    priority: ThreadPriority,

    context: CpuContext,
}

impl Thread {
    pub const fn new(
        thread_id: ThreadId,
        spawned_by: Option<ThreadId>,
        priority: ThreadPriority,
        context: CpuContext,
    ) -> Self {
        Self {
            thread_id,
            spawned_by,
            priority,
            context,
        }
    }

    pub const fn priority(&self) -> ThreadPriority {
        self.priority
    }

    pub const fn priority_index<const VEC_SIZE: usize>(&self) -> usize {
        assert!(VEC_SIZE.is_power_of_two());
        let step_size = ThreadPriority::MAX as usize / VEC_SIZE;

        (ThreadPriority::MAX - self.priority()) as usize / step_size
    }

    pub fn save_context(&mut self, ctx: CpuContext) {
        self.context = ctx;
    }

    pub fn context(&self) -> CpuContext {
        self.context.clone()
    }
}
