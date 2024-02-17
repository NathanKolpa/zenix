use core::{ptr::null, u64};

use bootinfo::*;
use essentials::FixedVec;

use crate::{
    bump_memory::BumpMemory,
    multiboot::{MultibootInfo, MultibootMMapEntry},
    paging::{align_down, align_up, PHYS_MEM_OFFSET},
    regions::{known_regions, stack_size},
};

pub fn setup_boot_info<'a>(
    mut bump_memory: BumpMemory,
    mmap: impl Iterator<Item = &'a MultibootMMapEntry> + Copy,
    kernel_module_region: MemoryRegion,
    multiboot_info: &'static MultibootInfo,
) -> u64 {
    let usable_memory = setup_mmap_info(&mut bump_memory, mmap, kernel_module_region);

    let boot_info = bump_memory.alloc_struct::<BootInfo>();

    boot_info.write(BootInfo {
        physical_memory_offset: PHYS_MEM_OFFSET as u64,
        stack_size: stack_size(),
        usable_heap: bump_memory.left_over_memory(), // TODO: add pre kernel to this.
        usable_memory,
        kernel_arguments: multiboot_info.cmdline(),
        bootloader_name: multiboot_info.boot_loader_name(),
        kernel_code: kernel_module_region,
        bump_memory: bump_memory.used_memory(),
    });

    boot_info.as_ptr() as u64
}

fn setup_mmap_info<'a>(
    bump_memory: &mut BumpMemory,
    mmap: impl Iterator<Item = &'a MultibootMMapEntry> + Copy,
    kernel_module_region: MemoryRegion,
) -> &'static [MemoryRegion] {
    let protected_regions_iter = [(kernel_module_region.start, kernel_module_region.size)]
        .into_iter()
        .chain(known_regions().map(|entry| (entry.start, entry.size)));

    let mut entries = FixedVec::<128, _>::new();

    for mm_entry in mmap.filter(|x| x.is_usable()) {
        let mut entry = new_region(mm_entry.addr(), mm_entry.size());

        split_entry_info(
            &mut entry,
            &mut entries,
            &mut protected_regions_iter.clone(),
        );

        if entry.size > 0 {
            entries.push(entry);
        }
    }

    let mut first_addr: *const MemoryRegion = null();

    for entry in entries.iter() {
        let slot = bump_memory.alloc_struct::<MemoryRegion>();

        if first_addr.is_null() {
            first_addr = slot.as_ptr();
        }

        slot.write(*entry);
    }

    unsafe { core::slice::from_raw_parts(first_addr, entries.len()) }
}

fn split_entry_info<const SIZE: usize>(
    entry: &mut MemoryRegion,
    entries: &mut FixedVec<SIZE, MemoryRegion>,
    protected_regions_iter: &mut (impl Iterator<Item = (u64, u64)> + Clone),
) {
    if entry.size <= 0 {
        return;
    }

    while let Some((region_start, region_size)) = protected_regions_iter.next() {
        let entry_end = entry.start + entry.size;
        let region_end = region_start + region_size;

        if entry_end >= region_start && entry.start <= region_end {
            entry.size = region_start.saturating_sub(entry.start);

            let room_left = entry_end.saturating_sub(region_end);

            if room_left <= 0 {
                continue;
            }

            let mut left_over_region = new_region(region_end, room_left);

            split_entry_info(
                &mut left_over_region,
                entries,
                &mut protected_regions_iter.clone(),
            );

            if left_over_region.size > 0 {
                entries.push(left_over_region);
            }
        }
    }
}

/// Align regions to save memory.
/// If a region doesn't fit in a page, it isn't used.
fn new_region(start: u64, size: u64) -> MemoryRegion {
    let aligned_start = align_up(start);
    let aligned_size = align_down(size.saturating_sub(aligned_start - start));

    MemoryRegion {
        start: aligned_start,
        size: aligned_size,
    }
}