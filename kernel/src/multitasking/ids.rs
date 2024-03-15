use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub type ProcessId = u32;
pub type AtomicProcessId = AtomicU32;

pub type ThreadId = u32;
pub type AtomicThreadId = AtomicU32;

pub struct AtomicProcThreadId {
    value: AtomicU64,
}

impl AtomicProcThreadId {
    fn decode(value: u64) -> (ProcessId, ThreadId) {
        ((value >> 32) as u32, value as u32)
    }

    const fn encode(process_id: ProcessId, thread_id: ThreadId) -> u64 {
        (process_id as u64) << 32 | thread_id as u64
    }

    pub const fn new(process_id: ProcessId, thread_id: ThreadId) -> Self {
        Self {
            value: AtomicU64::new(Self::encode(process_id, thread_id)),
        }
    }

    pub fn load(&self, order: Ordering) -> (ProcessId, ThreadId) {
        Self::decode(self.value.load(order))
    }

    pub fn compare_exchange(
        &self,
        current: (ProcessId, ThreadId),
        new: (ProcessId, ThreadId),
        success: Ordering,
        failure: Ordering,
    ) -> Result<(ProcessId, ThreadId), (ProcessId, ThreadId)> {
        let current = Self::encode(current.0, current.1);
        let new = Self::encode(new.0, new.1);

        self.value
            .compare_exchange(current, new, success, failure)
            .map(Self::decode)
            .map_err(Self::decode)
    }
}
