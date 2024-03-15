use essentials::address::VirtualAddress;

use crate::memory::map::MemoryMapper;

#[derive(Debug)]
pub enum MemoryViolation {
    NotMapped,
    InsufficientPrivilge,
    MalformedTable,
}

#[derive(Debug)]
pub enum MemoryAccess {
    Write,
    Read,
    InstructionFetch,
}

#[derive(Debug)]
pub struct PageFault {
    pub addr: VirtualAddress,
    pub instruction_pointer: VirtualAddress,
    pub access: MemoryAccess,
    pub violation: MemoryViolation,
}

pub enum MemoryErrorKind {
    NullPointer,
}

pub struct MemoryError {
    pub kind: MemoryErrorKind,
    pub fault: PageFault,
}

pub struct MemoryManager {
    mapper: MemoryMapper,
}

impl MemoryManager {
    pub const fn new(mapper: MemoryMapper) -> Self {
        Self { mapper }
    }

    pub fn handle_page_fault(&self, fault: PageFault) -> Result<(), MemoryError> {
        if fault.addr.is_null() {
            return Err(MemoryError {
                fault,
                kind: MemoryErrorKind::NullPointer,
            });
        }

        todo!()
    }
}
