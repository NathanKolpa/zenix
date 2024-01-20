use core::{fmt::Display, u16};

pub use errors::*;
pub use props::MemoryProperties;

use crate::{arch::x86_64::paging::*, util::FixedVec};
use crate::{
    memory::map::mapper::tree_display::MemoryMapTreeDisplay,
    util::address::{PhysicalAddress, VirtualAddress},
};

mod errors;
mod props;
mod tree_display;

struct NavigateCtx {
    entry: PageTableEntry,
    depth: u8,
    is_last_present_entry: bool,
    entry_index: u16,
}

const BORROW_BIT: u64 = 0;

/// The `MemoryMapper` struct manages the low-level mappings between physical and virtual addresses.
///
/// A MemoryMapper is responsible for managing a single [level 4 page
/// table](https://os.phil-opp.com/paging-introduction/). This means that thoughout the kernel
/// there may be multiple MemoryMappers. Typically one for each process.
///
/// # Ownership
///
/// The MemoryMapper adheres to the rust's concept of ownership. This is implemented by including a
/// single bit within each page that indicates whenether a page is owned (0) or borrowed (1).
/// Becaues of memory constraints it is not possible to refrence count these shared pages, this
/// means each shared page is inadvertently a "memory leak".
///
/// The positives of this model are that owned memory is automatically cleaned up after dropping an
/// instance of MemoryMapper. Futhermore, there might be some concurrency optmizations possible
/// because the MemoryMapper leaves this problem up to the caller.
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
    /// - The caller must ensure that `share_all` is called before the MemoryMapper gets dropped.
    pub unsafe fn from_active_page(global_offset: usize) -> Self {
        let l4_table = cr3::active_page();

        Self {
            global_offset,
            l4_table,
        }
    }

    /// Make all owned memory shared.
    ///
    /// # Memory Leak
    ///
    /// Prefoming this operation effectivly leaks all previously owned memory because it will not get dealloced
    /// after the `MemoryMapper` gets dropped.
    ///
    /// This is not a problem however, because the function intended use is to be called with only
    /// kernel memory. Because this memory never needs to be deallocated, this leaked memory will
    /// never cause any problem.
    pub fn share_all(&mut self) {
        let share_entry = |ctx: NavigateCtx| {
            let mut entry = ctx.entry;
            if entry.flags().custom::<BORROW_BIT>() {
                return None;
            }

            entry.set_flags(entry.flags().set_custom::<BORROW_BIT>(true));

            Some(entry)
        };

        unsafe {
            // For every entry in the l4 table, set the borrow bit to 1.
            // There is no need futher in the kernel to set the borrow bit in each child page.
            self.navigate_mut(
                VirtualAddress::null(),
                usize::MAX,
                Some(0),
                false,
                share_entry,
            )
            .unwrap()
        }
    }

    pub fn new_inherited_from_shared(&self) {}

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
        let flags = Self::props_to_flags(properties, true);
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

    fn navigate<E>(
        &self,
        start: VirtualAddress,
        size: usize,
        max_depth: Option<usize>,
        mut apply: impl FnMut(NavigateCtx) -> Result<(), E>,
    ) -> Result<(), E> {
        let max_depth = max_depth.unwrap_or(3).min(3);
        let end = start + size;

        let mut start_indices = start.indices();

        let mut table_stack = FixedVec::<4, &PageTable>::new();
        unsafe {
            table_stack.push(self.deref_l4_table());
        }

        let mut last_entry_index = -1;

        loop {
            let current_addr = VirtualAddress::from_indices(start_indices);

            if current_addr > end {
                break;
            }

            let table_index = table_stack.len();
            let Some(table) = table_stack.last_mut() else {
                break;
            };

            if last_entry_index == -1 {
                last_entry_index = table
                    .iter()
                    .enumerate()
                    .rfind(|(_, e)| e.flags().present())
                    .map(|(i, _)| i as i32)
                    .unwrap_or_default();
            }

            let entry_index = &mut start_indices[table_index - 1];

            let Some(entry_ref) = table.get(*entry_index as usize) else {
                table_stack.pop();
                *entry_index = 0;
                last_entry_index = -1;
                continue;
            };

            let entry = *entry_ref;

            let ctx = NavigateCtx {
                entry,
                depth: table_index as u8 - 1,
                is_last_present_entry: *entry_index == last_entry_index as u16,
                entry_index: *entry_index,
            };

            apply(ctx)?;

            *entry_index = entry_index.wrapping_add(1);

            let entry = *entry_ref;
            if entry.flags().present() && table_stack.len() <= max_depth && !entry.flags().huge() {
                let table = unsafe { self.deref_page_table(entry.addr()) };
                table_stack.push(table);
                last_entry_index = -1;
            }
        }

        Ok(())
    }

    unsafe fn navigate_mut(
        &mut self,
        start: VirtualAddress,
        size: usize,
        max_depth: Option<usize>,
        fail_on_unowned: bool,
        mut apply: impl FnMut(NavigateCtx) -> Option<PageTableEntry>,
    ) -> Result<(), ModifyMapError> {
        let max_depth = max_depth.unwrap_or(3).min(3);
        let end = start + size;

        let mut start_indices = start.indices();

        let mut table_stack = FixedVec::<4, &mut PageTable>::new();
        table_stack.push(self.deref_l4_table_mut());

        let mut last_entry_index = -1;

        loop {
            let current_addr = VirtualAddress::from_indices(start_indices);

            if current_addr > end {
                break;
            }

            let table_index = table_stack.len();
            let table_level = 4 - table_index + 1;
            let size = match table_level {
                2 => PageSize::Size2Mib,
                3 => PageSize::Size1Gib,
                _ => PageSize::Size4Kib,
            };

            let Some(table) = table_stack.last_mut() else {
                break;
            };

            if last_entry_index == -1 {
                last_entry_index = table
                    .iter()
                    .enumerate()
                    .rfind(|(_, e)| e.flags().present())
                    .map(|(i, _)| i as i32)
                    .unwrap_or_default();
            }

            let entry_index = &mut start_indices[table_index - 1];

            let Some(entry_ref) = table.get_mut(*entry_index as usize) else {
                table_stack.pop();
                *entry_index = 0;
                last_entry_index = -1;
                continue;
            };

            let entry = *entry_ref;

            if !entry.flags().custom::<BORROW_BIT>() {
                let ctx = NavigateCtx {
                    entry,
                    depth: table_index as u8 - 1,
                    entry_index: *entry_index,
                    is_last_present_entry: last_entry_index as u16 == *entry_index,
                };

                if let Some(new_entry) = apply(ctx) {
                    *entry_ref = new_entry;
                    // todo: invalidate the current table
                }
            } else if fail_on_unowned {
                return Err(ModifyMapError::NotOwned);
            }

            *entry_index = entry_index.wrapping_add(1);

            let entry = *entry_ref;
            if entry.flags().present() && table_stack.len() <= max_depth && !entry.flags().huge() {
                let table = self.deref_page_table_mut(entry.addr());
                table_stack.push(table);
                last_entry_index = -1;
            }
        }

        Ok(())
    }

    unsafe fn deref_l4_table(&self) -> &'static PageTable {
        self.deref_page_table(self.l4_table)
    }

    unsafe fn deref_l4_table_mut(&mut self) -> &'static mut PageTable {
        self.deref_page_table_mut(self.l4_table)
    }

    unsafe fn deref_page_table_mut(&mut self, addr: PhysicalAddress) -> &'static mut PageTable {
        let virt_addr = self.translate_table_frame(addr);
        &mut *virt_addr.as_mut_ptr()
    }

    unsafe fn deref_page_table(&self, addr: PhysicalAddress) -> &'static PageTable {
        let virt_addr = self.translate_table_frame(addr);
        &*virt_addr.as_ptr()
    }

    fn translate_table_frame(&self, phys: PhysicalAddress) -> VirtualAddress {
        (phys.as_usize() + self.global_offset).into()
    }

    const fn props_to_flags(props: MemoryProperties, present: bool) -> PageTableEntryFlags {
        PageTableEntryFlags::null()
            .set_present(present && props.readable())
            .set_writable(props.writable())
            .set_user_accessible(props.user())
            .set_no_exec(!props.executable())
    }

    pub fn tree_display(&self, max_depth: Option<usize>) -> impl Display + '_ {
        MemoryMapTreeDisplay::new(self, max_depth.unwrap_or(4) as u8)
    }
}
