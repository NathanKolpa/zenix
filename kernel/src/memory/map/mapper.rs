use core::{fmt::Display, u16};

pub use errors::*;
pub use props::MemoryProperties;

use crate::memory::alloc::FRAME_ALLOC;
use crate::memory::map::mapper::tree_display::MemoryMapTreeDisplay;

use essentials::address::{PhysicalAddress, VirtualAddress};
use essentials::FixedVec;
use x86_64::paging::*;

mod errors;
mod props;
mod tree_display;

struct NavigateCtx {
    entry: PageTableEntry,
    depth: u8,
    is_last_present_entry: bool,
    entry_index: u16,
    points_to_backing: bool,
    addr: VirtualAddress,
    size: usize,
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
        let share_entry = |ctx: NavigateCtx, _| -> Result<Option<PageTableEntry>, ModifyMapError> {
            let mut entry = ctx.entry;
            if entry.flags().custom::<BORROW_BIT>() || !entry.flags().present() {
                return Ok(None);
            }

            entry.set_flags(entry.flags().set_custom::<BORROW_BIT>(true));

            Ok(Some(entry))
        };

        unsafe {
            // For every entry in the l4 table, set the borrow bit to 1.
            // There is no need futher in the kernel to set the borrow bit in each child page.
            self.navigate_mut(
                VirtualAddress::null(),
                usize::MAX,
                Some(0),
                false,
                false,
                share_entry,
            )
            .unwrap();
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
    ) -> Result<(VirtualAddress, usize), NewMapError> {
        let flags = Self::props_to_flags(properties, true);

        // TODO: when FRAME_ALLOC::allocate returns none, we should go back and dealloc the created
        // frames.
        let mut total_size = 0;
        let alloc_missing = |ctx: NavigateCtx,
                             huge_size: Option<PageSize>|
         -> Result<Option<PageTableEntry>, NewMapError> {
            let mut entry = ctx.entry;
            let mut flags = entry.flags() | flags;

            let request_size = huge_size.unwrap_or(PageSize::Size4Kib);

            if !entry.flags().present() {
                if huge_size.is_some() {
                    flags = flags.set_huge(true);
                }

                let (frame_addr, _) = FRAME_ALLOC
                    .allocate_zeroed(request_size.as_usize())
                    .ok_or(NewMapError::OutOfFrames)?;

                entry.set_addr(frame_addr);
            }

            if ctx.points_to_backing {
                total_size += request_size.as_usize();
            }

            entry.set_flags(flags);
            Ok(Some(entry))
        };

        let start = unsafe { self.navigate_mut(address, size, None, true, false, alloc_missing) }?;

        Ok((start, total_size))
    }

    pub unsafe fn unmap(
        &mut self,
        _address: VirtualAddress,
        _size: usize,
    ) -> Result<(), ModifyMapError> {
        todo!()
    }

    /// Calculate the effective [`MemoryProperties`] on a given range of memory.
    pub fn effective_properties(
        &self,
        _address: VirtualAddress,
        _size: usize,
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

            if current_addr >= end {
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

                if let Some(prev) = table_index
                    .checked_sub(2)
                    .and_then(|prev_index| start_indices.get_mut(prev_index))
                {
                    *prev += 1;
                }

                continue;
            };

            let entry = *entry_ref;

            let ctx = NavigateCtx {
                addr: current_addr,
                entry,
                depth: table_index as u8 - 1,
                is_last_present_entry: *entry_index == last_entry_index as u16,
                entry_index: *entry_index,
                size: 4096 * 512usize.pow(4u32.saturating_sub(table_index as u32)),
                points_to_backing: (entry.flags().huge() && entry.flags().present())
                    || table_index == 4,
            };

            apply(ctx)?;

            let entry = *entry_ref;
            if entry.flags().present() && table_stack.len() <= max_depth && !entry.flags().huge() {
                let table = unsafe { self.deref_page_table(entry.addr()) };
                table_stack.push(table);
                last_entry_index = -1;
            } else {
                *entry_index += 1;
            }
        }

        Ok(())
    }

    unsafe fn navigate_mut<E>(
        &mut self,
        start: VirtualAddress,
        size: usize,
        max_depth: Option<usize>,
        fail_on_unowned: bool,
        fail_on_missing: bool,
        mut apply: impl FnMut(NavigateCtx, Option<PageSize>) -> Result<Option<PageTableEntry>, E>,
    ) -> Result<VirtualAddress, E>
    where
        E: From<ModifyMapError>,
    {
        let max_depth = max_depth.unwrap_or(3).min(3);
        let end = start + size;

        let mut start_indices = start.indices();
        let start_addr = VirtualAddress::from_indices(start_indices);

        let mut table_stack = FixedVec::<4, (&mut PageTable, PhysicalAddress)>::new();
        table_stack.push((self.deref_l4_table_mut(), self.l4_table));

        let mut last_entry_index = -1;

        loop {
            let current_addr = VirtualAddress::from_indices(start_indices);

            if current_addr >= end {
                break;
            }

            let table_index = table_stack.len();
            let table_level = 4 - table_index + 1;

            let entry_range = match table_level {
                2 => Some(PageSize::Size2Mib),
                3 => Some(PageSize::Size1Gib),
                1 => Some(PageSize::Size4Kib),
                _ => None,
            };

            let huge_size = entry_range.and_then(|size| {
                if size.as_usize() <= end.as_usize() - current_addr.as_usize() && table_level != 1 {
                    Some(size)
                } else {
                    None
                }
            });

            let Some((table, table_phys_addr)) = table_stack.last_mut() else {
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

                if let Some(prev) = table_index
                    .checked_sub(2)
                    .and_then(|prev_index| start_indices.get_mut(prev_index))
                {
                    *prev += 1;
                }
                continue;
            };

            let entry = *entry_ref;

            if !entry.flags().present() && fail_on_missing {
                return Err(ModifyMapError::NotMapped.into());
            }

            if !entry.flags().custom::<BORROW_BIT>() || !entry.flags().present() {
                let ctx = NavigateCtx {
                    addr: current_addr,
                    entry,
                    depth: table_index as u8 - 1,
                    entry_index: *entry_index,
                    is_last_present_entry: last_entry_index as u16 == *entry_index,
                    points_to_backing: huge_size.is_some() || table_index == 4,
                    size: 4096 * 512usize.pow(4u32.saturating_sub(table_index as u32)),
                };

                if let Some(new_entry) = apply(ctx, huge_size)? {
                    *entry_ref = new_entry;

                    // the entry has been modified in such a way that the tlb needs to be
                    // invalidated
                    if entry.addr() != new_entry.addr()
                        || !entry.flags().native_flags_eq(new_entry.flags())
                    {
                        cr3::flush_page(*table_phys_addr);
                    }
                }
            } else if fail_on_unowned {
                return Err(ModifyMapError::NotOwned.into());
            }

            let entry = *entry_ref;
            if entry.flags().present() && table_stack.len() <= max_depth && !entry.flags().huge() {
                let table = self.deref_page_table_mut(entry.addr());
                table_stack.push((table, entry.addr()));
                last_entry_index = -1;
            } else {
                *entry_index += 1;
            }
        }

        Ok(start_addr)
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

    pub fn tree_display(
        &self,
        start: VirtualAddress,
        size: usize,
        max_depth: Option<usize>,
    ) -> impl Display + '_ {
        MemoryMapTreeDisplay::new(self, max_depth.unwrap_or(3) as u8, start, size)
    }
}
