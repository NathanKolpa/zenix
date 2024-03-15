use core::sync::atomic::AtomicUsize;

use crate::memory::map::MemoryManager;

pub type ProcessId = u32;
pub type AtomicProcessId = AtomicUsize;

pub struct Process {
    id: ProcessId,
    manager: MemoryManager,
}
