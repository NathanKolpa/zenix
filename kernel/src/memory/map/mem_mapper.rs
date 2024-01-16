use crate::memory::alloc::FRAME_ALLOC;
use crate::util::address::VirtualAddress;

#[derive(Debug, Copy, Clone)]
pub struct MemoryProperties {
    writable: bool,
    readable: bool,
    kernel: bool,
    executable: bool,
}

impl MemoryProperties {
    pub const fn new(writable: bool, readable: bool, kernel: bool, executable: bool) -> Self {
        Self {
            writable,
            readable,
            kernel,
            executable,
        }
    }
}

pub enum NewMapError {
    NotOwned,
    OutOfFrames,
    AlreadyMapped,
}

pub enum ModifyMapError {
    NotOwned,
    NotMapped,
}

pub enum ReadMapError {
    NotMapped,
    InconsistentRange,
}

/// The `MemoryMapper` struct manages the low-level mappings between physical and virtual addresses.
///
/// # Ownership
///
pub struct MemoryMapper {}

impl MemoryMapper {
    /// Map a region of virtual memory.
    ///
    /// # Arguments
    ///
    /// * `address` - The starting virtual address of the memory region to be mapped.
    /// * `size` - The size of the memory region to be mapped, in bytes.
    ///
    /// Returns a `Result` where:
    /// * `Ok(size)` - Indicates successful mapping, returning the actual size of the mapped region.
    pub fn map(
        &mut self,
        address: VirtualAddress,
        size: usize,
        properties: MemoryProperties,
    ) -> Result<usize, NewMapError> {
        todo!()
    }

    pub unsafe fn unmap(&mut self, size: usize) -> Result<(), ModifyMapError> {
        todo!()
    }

    /// Calculate the effective [`MemoryProperties`] on a given range of memory.
    pub fn effective_properties(
        &self,
        address: VirtualAddress,
        size: usize,
    ) -> Result<MemoryProperties, ReadMapError> {
        todo!()
    }
}
