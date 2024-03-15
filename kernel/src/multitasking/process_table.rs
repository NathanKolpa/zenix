use essentials::spin::SpinLock;

use crate::{
    multitasking::process::{AtomicProcessId, ProcessId},
    utils::{InterruptGuard, ProcLocal},
};

pub struct ProcessTable {
    current_process: ProcLocal<AtomicProcessId>,
}
