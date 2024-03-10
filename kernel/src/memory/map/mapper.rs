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

#[derive(Debug)]
struct NavigateCtx {
    entry: PageTableEntry,
    depth: u8,
    is_last_present_entry: bool,
    entry_index: u16,
    points_to_backing: bool,
    addr: VirtualAddress,
    size: usize,
    is_empty: bool,
}

// TODO: the API that MemoryMapper provides is alright but my god the implementation sucks.
// TODO: the borrow bit could possible be simplified by giving a memory mapper a start + size and
// anything outside is not allowed.

const BORROW_BIT: u64 = 0;
const ALLOCATED_BIT: u64 = 1;

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
    root: bool,
}

impl MemoryMapper {
    pub const PAGE_SIZE: usize = 4096;

    /// Create a new `MemoryMapper` instance from the current `CR3` register value.
    ///
    /// # Safety
    ///
    /// - This function assumes the `global_offset` is valid for the current machine.
    /// - Creating more than one root mapper can result in UB.
    pub unsafe fn new_root_mapper(global_offset: usize) -> Self {
        let l4_table = cr3::active_page();

        Self {
            global_offset,
            l4_table,
            root: true,
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
        self.root = false;

        let share_entry = |ctx: NavigateCtx, _| -> Result<Option<PageTableEntry>, ModifyMapError> {
            let mut entry = ctx.entry;
            if entry.flags().custom::<BORROW_BIT>()
                || !entry.flags().present()
                || !ctx.points_to_backing
            {
                return Ok(None);
            }

            entry.set_flags(entry.flags().set_custom::<BORROW_BIT>(true));

            Ok(Some(entry))
        };

        unsafe {
            self.navigate_mut(
                VirtualAddress::null(),
                usize::MAX,
                None,
                false,
                false,
                false,
                share_entry,
            )
            .unwrap();
        }
    }

    pub fn new_inherited_from_shared(&self) {}

    pub fn unmap_all_owned(&mut self) {
        if self.root {
            panic!("unmap_all_owned() on a root MemoryMapper is extreemly bad.");
        }

        self.unmap_inner(VirtualAddress::null(), usize::MAX, true, false)
            .expect("unmap_all_owned should never fail")
    }

    /// Map a region of virtual memory.
    ///
    /// At no point should the kernel hold a refrence to the new mapping inner memory where unmapping it leads
    /// to UB. This way, [`Self::unmap`], [`Self::unmap_all_owned`] and [`Self::drop`] is safe.
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
        unsafe { self.map_inner(address, size, properties, None) }
    }

    /// Unmap a region of memory.
    ///
    /// When a operation is stopped due to an error, the original mappings will not be restored.
    /// Instead, the mappings will be in a unpredictable (but valid) state. The strictness of this
    /// API is meant to encourage the caller to be _correct_.
    ///
    /// This function is safe because the kernel should never hold refrences to memory created from
    /// [`Self::map`].
    ///
    /// # Arguments
    ///
    /// * `address` - The starting virtual address of the memory region to be unmapped.
    /// * `size` - The (maximum) size of the memory region to be unmapped, in bytes.
    ///
    /// # Returns a `Result` where:
    ///
    /// * `Err(ModifyMapError::OutOfBounds)` The requested size is smaller than can be possibly
    /// deallocated due to the page structure. Likely because the size is not aligned to the page
    /// size, or there is a huge page within the region.
    /// * `Err(ModifyMapError::NotOwned)` The region of memory contains pages that are not owned by
    /// the current MemoryMapper.
    /// * `Err(ModifyMapError::NotMapped)` The region of memory contains pages that are not mapped.
    /// * `Ok(())` The operation was successfull.
    pub fn unmap(&mut self, address: VirtualAddress, size: usize) -> Result<(), ModifyMapError> {
        self.unmap_inner(address, size, false, true)
    }

    fn unmap_inner(
        &mut self,
        address: VirtualAddress,
        size: usize,
        lax: bool,
        fail_on_out_of_bounds: bool,
    ) -> Result<(), ModifyMapError> {
        let mut total_size = 0;

        let dealloc = |ctx: NavigateCtx,
                       huge_size: Option<PageSize>|
         -> Result<Option<PageTableEntry>, ModifyMapError> {
            let mut entry = ctx.entry;

            if (!ctx.points_to_backing && !ctx.is_empty)
                || !entry.flags().present()
                || entry.flags().custom::<BORROW_BIT>()
            {
                return Ok(None);
            }

            let request_size = huge_size.unwrap_or(PageSize::Size4Kib);

            if request_size.as_usize() > size && fail_on_out_of_bounds {
                return Err(ModifyMapError::OutOfBounds);
            }

            if entry.flags().custom::<ALLOCATED_BIT>() {
                // Safety:
                // Owned pages *should* be using allocated using `FRAME_ALLOC`.
                unsafe { FRAME_ALLOC.deallocate(entry.addr()) };
            }

            total_size += request_size.as_usize();

            entry.set_flags(PageTableEntryFlags::null());
            entry.set_addr(PhysicalAddress::null());

            Ok(Some(entry))
        };

        unsafe { self.navigate_mut(address, size, None, !lax, !lax, true, dealloc) }?;

        Ok(())
    }

    unsafe fn map_inner(
        &mut self,
        address: VirtualAddress,
        size: usize,
        properties: MemoryProperties,
        mut phys: Option<PhysicalAddress>,
    ) -> Result<(VirtualAddress, usize), NewMapError> {
        let flags = Self::props_to_flags(properties, true);

        let mut total_size = 0;
        let alloc_missing = |ctx: NavigateCtx,
                             huge_size: Option<PageSize>|
         -> Result<Option<PageTableEntry>, NewMapError> {
            let mut entry = ctx.entry;
            let mut flags = (entry.flags() | flags)
                .set_no_cache(false)
                .set_no_exec(false);

            let request_size = huge_size.unwrap_or(PageSize::Size4Kib);

            if ctx.points_to_backing && entry.flags().present() {
                let unchanged = phys.map(|phys| phys == entry.addr()).unwrap_or_default();

                if !unchanged {
                    return Err(NewMapError::AlreadyMapped);
                }
            }

            if !entry.flags().present() {
                if huge_size.is_some() {
                    flags = flags.set_huge(true);
                }

                let mut frame_addr = None;

                if ctx.points_to_backing {
                    if let Some(phys) = phys.as_mut() {
                        frame_addr = Some(*phys);
                        *phys += request_size.as_usize();
                        total_size += request_size.as_usize();

                        if properties.mmio() {
                            flags = flags.set_no_cache(true);
                        }

                        if !properties.executable() {
                            flags = flags.set_no_exec(true);
                        }
                    }
                }

                let frame_addr = match frame_addr {
                    Some(a) => a,
                    None => {
                        let (frame_addr, _) = FRAME_ALLOC
                            .allocate_zeroed(request_size.as_usize())
                            .ok_or(NewMapError::OutOfFrames)?;

                        flags = flags.set_custom::<ALLOCATED_BIT>(true);

                        frame_addr
                    }
                };

                entry.set_addr(frame_addr);
            }

            entry.set_flags(flags);
            Ok(Some(entry))
        };

        let navigation_result =
            self.navigate_mut(address, size, None, true, false, false, alloc_missing);

        let start = match navigation_result {
            Ok(start) => start,
            Err(NewMapError::OutOfFrames) => {
                self.unmap_inner(address, total_size, false, false)
                    .expect("dealloc should not fail on memory created by map()");

                return Err(NewMapError::OutOfFrames);
            }
            Err(e) => return Err(e),
        };

        Ok((start, total_size))
    }

    /// Identity map memory
    ///
    /// Returns a `Result` where:
    /// * `Ok(address, size)` - Indicates successful mapping, returning the actual size and address of the mapped region.
    /// * Err(NotOwned) - Tried to map to an unowned region of memory OR the region clashes with
    /// memory in [`FRAME_ALLOC`]
    pub fn identity_map(
        &mut self,
        address: PhysicalAddress,
        size: usize,
        properties: MemoryProperties,
    ) -> Result<(VirtualAddress, usize), NewMapError> {
        assert!(self.root);

        let aligned_addr = address.align_down(Self::PAGE_SIZE);

        let aligned_size = VirtualAddress::align_ptr_up(
            size + (address - aligned_addr).as_usize(),
            Self::PAGE_SIZE,
        );

        if FRAME_ALLOC.clashes(aligned_addr, aligned_size) {
            return Err(NewMapError::NotOwned);
        }

        // Safety: no backing with 2 mappings can be created because:
        // - It does not clash with FRAME_ALLOC
        // - There are no other MemoryMapper objects as asserted by self.root == true
        unsafe {
            self.map_inner(
                VirtualAddress::from(aligned_addr.as_usize()),
                aligned_size,
                properties,
                Some(aligned_addr),
            )
        }
    }

    /// Get mapping info about a particular address.
    ///
    /// # Return a `Result` where:
    ///
    /// * Ok() A union constisting of:
    ///   * [`MemoryProperties`] The effective memory properties of the address.
    ///   * [`PhysicalAddress`] The physical address of the mapping.
    ///   * [`usize`] The size of the page in bytes
    pub fn mapping_info(
        &self,
        address: VirtualAddress,
    ) -> Result<(MemoryProperties, PhysicalAddress, usize), ReadMapError> {
        let mut addr = None;
        let mut flags = PageTableEntryFlags::null();
        let mut size = 0;

        let mut apply = |ctx: NavigateCtx| -> Result<(), ReadMapError> {
            if ctx.points_to_backing {
                addr = Some(ctx.entry.addr());
                size += ctx.size;
            }

            if ctx.depth == 0 {
                flags = ctx.entry.flags();
            } else {
                flags = flags & ctx.entry.flags();
            }

            Ok(())
        };

        self.navigate(address, Self::PAGE_SIZE, None, &mut apply)?;

        let Some(addr) = addr else {
            return Err(ReadMapError::NotMapped);
        };

        if !flags.present() {
            return Err(ReadMapError::NotMapped);
        }

        Ok((Self::flags_to_props(flags), addr, size))
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
                size: Self::PAGE_SIZE * 512usize.pow(4u32.saturating_sub(table_index as u32)),
                points_to_backing: (entry.flags().huge() && entry.flags().present())
                    || table_index == 4,
                is_empty: false,
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
        revisit_parent: bool,
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
        let mut revisit = false;

        let mut current_addr = VirtualAddress::from_indices(start_indices);
        loop {
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

                if !revisit_parent {
                    if let Some(prev) = table_index
                        .checked_sub(2)
                        .and_then(|prev_index| start_indices.get_mut(prev_index))
                    {
                        *prev += 1;
                    }
                } else {
                    revisit = true;
                }

                current_addr = VirtualAddress::from_indices(start_indices);
                continue;
            };

            let entry = *entry_ref;

            if !entry.flags().present() && fail_on_missing && current_addr != end {
                return Err(ModifyMapError::NotMapped.into());
            }

            if !entry.flags().custom::<BORROW_BIT>() || !entry.flags().present() {
                let mut is_empty = false;

                if entry.flags().present() && !entry.flags().huge() && table_index <= max_depth {
                    let table = self.deref_page_table(entry.addr());
                    is_empty = !table.iter().any(|e| e.flags().present());
                }

                let ctx = NavigateCtx {
                    addr: current_addr,
                    entry,
                    depth: table_index as u8 - 1,
                    entry_index: *entry_index,
                    is_last_present_entry: last_entry_index as u16 == *entry_index,
                    points_to_backing: (entry.flags().huge() && entry.flags().present())
                        || table_index == 4,
                    size: Self::PAGE_SIZE * 512usize.pow(4u32.saturating_sub(table_index as u32)),
                    is_empty,
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
            if entry.flags().present()
                && table_stack.len() <= max_depth
                && !entry.flags().huge()
                && !revisit
            {
                let table = self.deref_page_table_mut(entry.addr());
                table_stack.push((table, entry.addr()));
                last_entry_index = -1;
            } else {
                revisit = false;
                *entry_index += 1;

                current_addr = VirtualAddress::from_indices(start_indices);

                if current_addr >= end {
                    break;
                }
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
            .set_global(props.kernel())
            .set_no_cache(props.mmio())
    }

    const fn flags_to_props(flags: PageTableEntryFlags) -> MemoryProperties {
        MemoryProperties::new(
            flags.writable(),
            flags.present(),
            !flags.user_accessible(),
            !flags.noexec(),
            flags.no_cache(),
        )
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

impl Drop for MemoryMapper {
    fn drop(&mut self) {
        self.unmap_all_owned();
    }
}
