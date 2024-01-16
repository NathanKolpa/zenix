pub use errors::*;
pub use props::MemoryProperties;

use crate::arch::x86_64::paging::cr3;
use crate::util::address::{PhysicalAddress, VirtualAddress};

mod errors;
mod props;

/// The `MemoryMapper` struct manages the low-level mappings between physical and virtual addresses.
///
/// # Ownership
///
pub struct MemoryMapper {
    l4_table: PhysicalAddress,
    global_offset: usize,
}

impl MemoryMapper {
    /// Create a new `MemoryMapper` instance from the current `CR3` register value.
    ///
    /// # Safety
    ///
    /// - This function assumes the `global_offset` is valid for the current machine.
    /// - Creating more than one instance though this function can lead to UB because borrow checking is not preformed.
    pub unsafe fn from_active_page(global_offset: usize) -> Self {
        let l4_table = cr3::active_page();

        Self {
            global_offset,
            l4_table,
        }
    }

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
