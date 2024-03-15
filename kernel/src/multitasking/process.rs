use core::sync::atomic::AtomicUsize;

use crate::memory::map::MemoryManager;

pub type ProcessId = u32;
pub type AtomicProcessId = AtomicUsize;

pub struct Process {
    process_id: ProcessId,
    manager: MemoryManager,
}

impl Process {
    pub fn new(process_id: ProcessId, manager: MemoryManager) -> Self {
        assert_ne!(0, process_id);

        Self {
            process_id,
            manager,
        }
    }
}
