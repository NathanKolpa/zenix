use essentials::PanicOnce;

use crate::{multitasking::process::AtomicProcessId, utils::ProcLocal};

pub struct ProcessTable {
    current_process: PanicOnce<ProcLocal<AtomicProcessId>>,
}

impl ProcessTable {
    pub const fn new() -> Self {
        Self {
            current_process: PanicOnce::new(),
        }
    }

    pub fn init(&self) {
        self.current_process
            .initialize_with(ProcLocal::new(|| AtomicProcessId::new(0)));
    }
}

pub static PROCESS_TABLE: ProcessTable = ProcessTable::new();
