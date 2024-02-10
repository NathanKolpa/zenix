use x86_64::paging::{PageSize, PageTable, PageTableEntry, PageTableEntryFlags};

use crate::{
    bump_memory::BumpMemory,
    multiboot::{MultibootMMapEntry, MultibootModule},
};

pub const PHYS_MEM_OFFSET: u64 = 250_0000_0000_0000;

extern "C" {
    static BUMP_MEMORY_START: u8;
    static BUMP_MEMORY_END: u8;

    static PRE_KERNEL_START: u8;
    static PRE_KERNEL_END: u8;

    static STACK_START: u8;
    static STACK_END: u8;
}

/// Setup paging before entering long mode.
///
/// The resulting page table contains the following mappings:
/// - All physical memory is mapped as: virtual address + [`PSHY_MEM_OFFSET`].
/// - The kernel, pre-kernel and bumb memory is [identity mapped
/// ](https://wiki.osdev.org/Identity_Paging).
///
/// When possible, huge pages are used in favour of smaller 4Kib pages.
///
/// This fuction returns the (both virtual and physical) address of the l4 page.
pub unsafe fn setup_paging<'a>(
    bump_memory: &mut BumpMemory,
    memory_map: impl Iterator<Item = &'a MultibootMMapEntry>,
    kernel_mod: &MultibootModule,
) -> u32 {
    let l4_table = new_empty_page_table(bump_memory);

    for mm_entry in memory_map.filter(|e| e.is_usable()) {
        let addr = mm_entry.addr();
        let len = mm_entry.len();

        map_phys_range(bump_memory, l4_table, PHYS_MEM_OFFSET, true, 3, addr, len);
    }

    let pre_start = unsafe { &PRE_KERNEL_START as *const _ as u64 };
    let pre_end = unsafe { &PRE_KERNEL_END as *const _ as u64 };
    let pre_len = pre_end - pre_start;

    let bump_start = unsafe { &BUMP_MEMORY_START as *const _ as u64 };
    let bump_end = unsafe { &BUMP_MEMORY_END as *const _ as u64 };
    let bump_len = bump_end - bump_start;

    let stack_start = unsafe { &STACK_START as *const _ as u64 };
    let stack_end = unsafe { &STACK_END as *const _ as u64 };
    let stack_len = stack_end - stack_start;

    map_phys_range(bump_memory, l4_table, 0, true, 3, bump_start, bump_len);
    map_phys_range(bump_memory, l4_table, 0, true, 3, stack_start, stack_len);
    map_phys_range(bump_memory, l4_table, 0, false, 3, pre_start, pre_len);

    l4_table as *const _ as u32
}

fn map_phys_range(
    bump_memory: &mut BumpMemory,
    parent: &mut PageTable,
    offset: u64,
    no_exec: bool,
    level: u8,
    mut start: u64,
    mut len: u64,
) -> (u64, u64) {
    const PAGE_SIZE: u64 = 4096;

    // align up
    start = (start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

    // aling down
    len = len & !(PAGE_SIZE - 1);

    let level_page_size = match level {
        2 => Some(PageSize::Size1Gib),
        1 => Some(PageSize::Size2Mib),
        0 => Some(PageSize::Size4Kib),
        _ => None,
    };

    let flags = PageTableEntryFlags::null()
        .set_no_exec(no_exec)
        .set_global(true)
        .set_present(true)
        .set_writable(true);

    // map the memory backing to the page table.
    if let Some(level_page_size) = level_page_size.map(|x| x.as_usize() as u64) {
        while len >= level_page_size {
            let index = virt_addr_to_index(level, start + offset);

            if index >= 512 {
                return (start, len);
            }

            let leaf_page_flags = flags.set_huge(level > 0);

            let entry = PageTableEntry::new_u64_addr(leaf_page_flags, start);

            parent[index as usize] = entry;

            start += level_page_size;
            len -= level_page_size;
        }
    }

    // create smaller page tables for when there is memory left.
    // l1 pages should implicitly not enter this loop.
    while len >= PAGE_SIZE {
        let index = virt_addr_to_index(level, start + offset);

        if index >= 512 {
            break;
        }

        let table = if !parent[index as usize].flags().present() {
            let new_table = new_empty_page_table(bump_memory);
            let entry = PageTableEntry::new_u64_addr(flags, new_table as *const _ as u64);

            parent[index as usize] = entry;

            new_table
        } else {
            let existing_entry = &mut parent[index as usize];
            let existing_flags = existing_entry.flags();

            if !no_exec && existing_flags.noexec() {
                existing_entry.set_flags(existing_flags.set_no_exec(false));
            }

            let table_addr = existing_entry.addr_u64() as *mut PageTable;
            unsafe { &mut *table_addr }
        };

        (start, len) = map_phys_range(bump_memory, table, offset, no_exec, level - 1, start, len);
    }

    (start, len)
}

fn virt_addr_to_index(level: u8, mut addr: u64) -> u16 {
    fn truncate_index(value: u64) -> u16 {
        (value % 512) as u16
    }

    addr >>= 12 + 9 * level;

    truncate_index(addr)
}

fn new_empty_page_table(bump_memory: &mut BumpMemory) -> &'static mut PageTable {
    let table = bump_memory.alloc_struct::<PageTable>();
    let table = unsafe { table.assume_init_mut() };
    table.zero();
    table
}
