use core::{i64, u64, u8, usize};

use elf::{ElfReadError, ElfReader, RelocationEntryKind, SectionKind};
use essentials::FixedVec;
use x86_64::paging::{PageSize, PageTable, PageTableEntry, PageTableEntryFlags};

use crate::{
    bump_memory::BumpMemory,
    multiboot::{MultibootMMapEntry, MultibootModule},
    vga::{VGA_ADDR, VGA_LEN},
};

pub const PHYS_MEM_OFFSET: i64 = 250_0000_0000_0000;
const PAGE_SIZE: u64 = 4096;

extern "C" {
    static BUMP_MEMORY_START: u8;
    static BUMP_MEMORY_END: u8;

    static PRE_KERNEL_START: u8;
    static PRE_KERNEL_END: u8;

    static STACK_START: u8;
    static STACK_END: u8;
}

pub enum PagingSetupError {
    OverlappingKernel,
    ElfNotAligned,
    ReloTableKind,
    Bits32Unsupported,
    NoEntryPoint,
    UnsupportedArch,
    ElfError(ElfReadError),
}

impl PagingSetupError {
    pub fn as_str(&self) -> &'static str {
        match self {
            PagingSetupError::OverlappingKernel => "the kernel overlaps other memory",
            PagingSetupError::ElfError(e) => e.as_str(),
            PagingSetupError::ElfNotAligned => "Elf file not aligned with 4KiB",
            PagingSetupError::ReloTableKind => "Relocation tables are not fully supported",
            PagingSetupError::Bits32Unsupported => "32-bit kernels are not supported",
            PagingSetupError::NoEntryPoint => "No entry point",
            PagingSetupError::UnsupportedArch => "Architecture not supported",
        }
    }
}

impl From<ElfReadError> for PagingSetupError {
    fn from(value: ElfReadError) -> Self {
        Self::ElfError(value)
    }
}

/// Setup paging before entering long mode.
///
/// The resulting page table contains the following mappings:
/// - All physical memory is mapped as: virtual address + [`PSHY_MEM_OFFSET`].
/// - The pre-kernel and bumb memory is [identity mapped
/// ](https://wiki.osdev.org/Identity_Paging).
/// - The kernel is mapped to the offset in the ELF file. The actual content is not copied, but mapped
/// to the (in memory) ELF file.
///
/// When possible, huge pages are used in favour of smaller 4Kib pages.
///
/// This fuction returns the (identity mapped) address of the l4 page and entrypoint of the kernel.
pub fn setup_paging<'a>(
    bump_memory: &mut BumpMemory,
    memory_map: impl Iterator<Item = &'a MultibootMMapEntry>,
    kernel_module: &mut MultibootModule,
) -> Result<(u32, u64), PagingSetupError> {
    let l4_table = new_empty_page_table(bump_memory);

    let mut map = |offset: i64, no_exec: bool, writable: bool, start: u64, len: u64| {
        map_phys_range(
            bump_memory,
            l4_table,
            offset,
            0,
            no_exec,
            writable,
            3,
            start,
            len,
        );
    };

    // phys to virt mappings

    for mm_entry in memory_map.filter(|e| e.is_usable()) {
        let addr = mm_entry.addr();
        let len = mm_entry.len();

        map(PHYS_MEM_OFFSET, true, true, addr, len);
    }

    // static mappings
    let pre_start = unsafe { &PRE_KERNEL_START as *const _ as u64 };
    let pre_end = unsafe { &PRE_KERNEL_END as *const _ as u64 };
    let pre_len = pre_end - pre_start;

    let bump_start = unsafe { &BUMP_MEMORY_START as *const _ as u64 };
    let bump_end = unsafe { &BUMP_MEMORY_END as *const _ as u64 };
    let bump_len = bump_end - bump_start;

    let stack_start = unsafe { &STACK_START as *const _ as u64 };
    let stack_end = unsafe { &STACK_END as *const _ as u64 };
    let stack_len = stack_end - stack_start;

    let vga_start = VGA_ADDR.as_u64();
    let vga_len = VGA_LEN as u64;

    map(0, true, true, bump_start, bump_len);
    map(0, true, true, stack_start, stack_len);
    map(0, false, false, pre_start, pre_len);
    map(0, true, true, vga_start, vga_len);

    // kernel mappings
    let elf_start = kernel_module.addr() as u64;
    let elf_end = elf_start + kernel_module.len() as u64;

    if elf_start % PAGE_SIZE != 0 {
        return Err(PagingSetupError::ElfNotAligned);
    }

    let protected_regions = [
        (pre_start, pre_end),
        (bump_start, bump_end),
        (stack_start - PAGE_SIZE, stack_end + PAGE_SIZE),
        (elf_start, elf_end),
    ];

    let entry_point = map_kernel(bump_memory, &protected_regions, kernel_module, l4_table)?;

    Ok((l4_table as *const _ as u32, entry_point))
}

fn map_kernel(
    bump_memory: &mut BumpMemory,
    protected_regions: &[(u64, u64)],
    kernel_module: &mut MultibootModule,
    l4_table: &mut PageTable,
) -> Result<u64, PagingSetupError> {
    let raw_kernel_elf = kernel_module.bytes();
    let kernel_ident = ElfReader::new(raw_kernel_elf)?;
    let kernel_elf = match kernel_ident.header()? {
        elf::ArchHeaderReader::Bits64(k) => k,
        elf::ArchHeaderReader::Bits32(_) => {
            return Err(PagingSetupError::Bits32Unsupported);
        }
    };

    let Some(entry_point) = kernel_elf.entry_point() else {
        return Err(PagingSetupError::NoEntryPoint);
    };

    match kernel_elf.arch() {
        elf::Arch::Archx86_64 => {}
        _ => {
            return Err(PagingSetupError::UnsupportedArch);
        }
    }

    let elf_start = kernel_elf.elf_start().as_u64();

    let relo_table = kernel_elf.relocation_table()?;

    for program_header in kernel_elf.program_headers()? {
        let virt_addr = program_header.addr();
        let mem_len = program_header.memory_size();
        let virt_end = virt_addr + mem_len;

        let phys_offset = elf_start as i64 + program_header.data_offset() as i64 - virt_addr as i64;

        for (region_start, region_end) in protected_regions {
            if virt_end >= *region_start && virt_addr <= *region_end {
                return Err(PagingSetupError::OverlappingKernel);
            }
        }

        if program_header.kind() != SectionKind::Load {
            continue;
        }

        let flags = program_header.flags();

        if !flags.readable() {
            continue;
        }

        let no_exec = false; // !flags.executable();
        let writable = flags.writable();

        // parts that can be mapped the raw elf module.
        let file_len = program_header.file_size();

        map_phys_range(
            bump_memory,
            l4_table,
            0,
            phys_offset,
            no_exec,
            writable,
            3,
            virt_addr,
            file_len,
        );

        // The size in memory is larger then the size in the file.
        // This means that whatever is not placed in the file, should be mapped to zero.
        if mem_len > program_header.file_size() {
            let zero_addr = virt_addr + program_header.file_size();
            let zero_len = program_header.memory_size() - program_header.file_size();

            let zero_addr_aligned = align_down(zero_addr);
            let zero_len_aligned = align_up(zero_len);

            let alignment_offset = (zero_addr - zero_addr_aligned) as usize;
            let backing = bump_memory.alloc(zero_len as usize + alignment_offset);
            let backing_offset = backing.as_ptr() as i64 - zero_addr as i64;

            map_phys_range(
                bump_memory,
                l4_table,
                0,
                backing_offset,
                no_exec,
                writable,
                3,
                zero_addr_aligned,
                zero_len_aligned,
            );

            // Copy whatever is between zero addr and aligned zero addr into the backing.
            // The rest should be zeros.
            let end_index = program_header.file_size() as usize;
            let start_index = end_index - alignment_offset;

            let left_over_data = &program_header.bytes()?[start_index..end_index];
            let (left_over_backing, zero_backing) = backing.split_at_mut(alignment_offset);

            left_over_backing.copy_from_slice(left_over_data);

            for byte in zero_backing {
                *byte = 0;
            }
        }
    }

    // Final step, in LOAD sections apply relocaitons.
    // The relocation table is borrowed, so we have to copy the table first.
    let mut relo_table_copy = FixedVec::<128, _>::new();
    for program_header in kernel_elf.program_headers()? {
        if program_header.kind() != SectionKind::Load {
            continue;
        }

        let header_addr = program_header.addr();

        let relo_entries = relo_table
            .iter()
            .flat_map(|t| t.into_iter())
            .filter(|entry| {
                entry.offset() >= header_addr
                    && entry.offset() <= header_addr + program_header.memory_size()
            });

        for entry in relo_entries {
            if entry.offset() > header_addr + program_header.file_size() {
                // TODO: allow for relocation in .bss sections
                return Err(PagingSetupError::ReloTableKind);
            }

            if entry.kind() != RelocationEntryKind::Relative {
                return Err(PagingSetupError::ReloTableKind);
            }

            let index = entry.offset() - program_header.addr() + program_header.data_offset();
            relo_table_copy.push((entry.addend(), index));
        }
    }

    let elf_bytes = kernel_module.bytes_mut();
    for (addend, index) in relo_table_copy.iter() {
        let index = *index as usize;
        let value_bytes = addend.to_ne_bytes();
        let dest_bytes = &mut elf_bytes[(index)..(index + value_bytes.len())];
        dest_bytes.copy_from_slice(&value_bytes);
    }

    Ok(entry_point)
}

fn map_phys_range(
    bump_memory: &mut BumpMemory,
    parent: &mut PageTable,
    offset: i64,
    phys_offset: i64,
    no_exec: bool,
    writable: bool,
    level: u8,
    mut start: u64,
    mut len: u64,
) -> (u64, u64) {
    let unaligned_start = start;
    start = align_down(start);

    len = align_up(len + (unaligned_start - start));

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
        .set_writable(writable);

    while len >= PAGE_SIZE {
        let addr = align_down((start as i64 + offset) as u64);

        let index = virt_addr_to_index(level, addr);

        if index >= 512 {
            break;
        }

        if let Some(level_page_size) = level_page_size.map(|x| x.as_usize() as u64) {
            if len >= level_page_size && addr % level_page_size == 0 {
                let existing_flags = parent[index as usize].flags();
                let mut merged_flags = flags.set_huge(level > 0);

                let phys_addr = (start as i64 + phys_offset) as u64;

                if existing_flags.present() {
                    merged_flags = merged_flags | existing_flags;
                }

                let entry = PageTableEntry::new_u64_addr(merged_flags, phys_addr);

                parent[index as usize] = entry;

                start += level_page_size;
                len -= level_page_size;

                if index == 511 {
                    break;
                }

                continue;
            }
        }

        if level > 0 {
            let table = if !parent[index as usize].flags().present() {
                let new_table = new_empty_page_table(bump_memory);
                let entry = PageTableEntry::new_u64_addr(flags, new_table as *const _ as u64);

                parent[index as usize] = entry;

                new_table
            } else {
                let existing_entry = &mut parent[index as usize];
                let mut existing_flags = existing_entry.flags();

                if existing_flags.huge() {
                    panic!("Tried to decent into huge page");
                }

                if !no_exec && existing_flags.noexec() {
                    existing_flags = existing_flags.set_no_exec(false);
                }

                if writable && !existing_flags.writable() {
                    existing_flags = existing_flags.set_writable(true);
                }

                existing_entry.set_flags(existing_flags);

                let table_addr = existing_entry.addr_u64() as *mut PageTable;
                unsafe { &mut *table_addr }
            };

            (start, len) = map_phys_range(
                bump_memory,
                table,
                offset,
                phys_offset,
                no_exec,
                writable,
                level - 1,
                start,
                len,
            );
        }
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

fn align_up(addr: u64) -> u64 {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

fn align_down(addr: u64) -> u64 {
    addr & !(PAGE_SIZE - 1)
}
