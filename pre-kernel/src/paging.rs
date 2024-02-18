use elf::{ElfHeaderReader, ElfReadError, ElfReader, RelocationEntryKind, SectionKind};
use x86_64::paging::{PageSize, PageTable, PageTableEntry, PageTableEntryFlags};

use crate::{
    bump_memory::BumpMemory,
    multiboot::{MultibootMMapEntry, MultibootModule},
    regions::known_regions,
};

pub const PHYS_MEM_OFFSET: i64 = 1024 * 1024 * 1024 * 1024 * 60; // 60 TiB
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
    kernel: &MultibootModule,
) -> Result<&'static mut PageTable, PagingSetupError> {
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
        let len = mm_entry.size();

        map(PHYS_MEM_OFFSET, true, true, addr, len);
    }

    let known_regions = known_regions();

    for region in known_regions {
        map(
            0,
            !region.executable,
            region.writable,
            region.start,
            region.size,
        );
    }

    map(0, true, true, kernel.addr() as u64, kernel.len() as u64);

    Ok(l4_table)
}

fn map_sections<'a>(
    bump_memory: &mut BumpMemory,
    l4_table: &mut PageTable,
    raw_kernel_elf: &'a [u8],
) -> Result<ElfHeaderReader<'a, u64>, PagingSetupError> {
    let kernel_ident = ElfReader::new(raw_kernel_elf)?;

    let kernel_elf = match kernel_ident.header()? {
        elf::ArchHeaderReader::Bits64(k) => k,
        elf::ArchHeaderReader::Bits32(_) => {
            return Err(PagingSetupError::Bits32Unsupported);
        }
    };

    match kernel_elf.arch() {
        elf::Arch::Archx86_64 => {}
        _ => {
            return Err(PagingSetupError::UnsupportedArch);
        }
    }

    let elf_start = kernel_elf.elf_start().as_u64();
    if elf_start % PAGE_SIZE != 0 {
        return Err(PagingSetupError::ElfNotAligned);
    }

    for program_header in kernel_elf.program_headers()? {
        let virt_addr = program_header.addr();
        let mem_len = program_header.memory_size();
        let virt_end = virt_addr + mem_len;

        let phys_offset = elf_start as i64 + program_header.data_offset() as i64 - virt_addr as i64;

        for region in known_regions() {
            if virt_end >= region.start && virt_addr <= region.start + region.size {
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

        let no_exec = !flags.executable();
        let writable = flags.writable();

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
            let bss_addr = virt_addr + program_header.file_size();
            let bss_len = program_header.memory_size() - program_header.file_size();

            let bss_addr_aligned = align_down(bss_addr);

            let alignment_offset = (bss_addr - bss_addr_aligned) as usize;
            let bss_len_aligned = align_up(bss_len + alignment_offset as u64);

            let backing = bump_memory.alloc_aligned(bss_len_aligned as usize, PAGE_SIZE as usize);
            let backing_offset = backing.as_ptr() as i64 - bss_addr_aligned as i64;

            map_phys_range(
                bump_memory,
                l4_table,
                0,
                backing_offset,
                no_exec,
                writable,
                3,
                bss_addr_aligned,
                bss_len_aligned,
            );

            // The .bss section is aligned down, this mean that there could be some data left that
            // is not part of the .bss section. This menas that what ever is left needs to be
            // copied over. The rest should be initialized to 0.
            let end_index = program_header.file_size() as usize;
            let start_index = end_index - alignment_offset;

            let (left_over_backing, zero_backing) = backing.split_at_mut(alignment_offset);

            let left_over_data = &program_header.bytes()?[start_index..end_index];
            left_over_backing.copy_from_slice(left_over_data);

            for byte in zero_backing {
                *byte = 0;
            }
        }
    }

    Ok(kernel_elf)
}

unsafe fn apply_relocations(kernel_elf: ElfHeaderReader<'_, u64>) -> Result<(), PagingSetupError> {
    let Some(relo_table) = kernel_elf.relocation_table()? else {
        return Ok(());
    };

    for relo_entry in relo_table {
        if relo_entry.kind() != RelocationEntryKind::Relative {
            return Err(PagingSetupError::ReloTableKind);
        }

        core::ptr::write(relo_entry.offset() as *mut u64, relo_entry.addend());
    }

    Ok(())
}

// TODO
fn unmap_module(kernel: &MultibootModule) -> Result<(), PagingSetupError> {
    Ok(())
}

// TODO
fn protect_relocations(kernel: &MultibootModule) -> Result<(), PagingSetupError> {
    Ok(())
}

pub unsafe fn map_kernel(
    bump_memory: &mut BumpMemory,
    kernel: &MultibootModule,
    l4_table: &mut PageTable,
) -> Result<u64, PagingSetupError> {
    let raw_kernel_elf =
        core::slice::from_raw_parts((kernel.addr()) as *const u8, kernel.len() as usize);

    let kernel_elf = map_sections(bump_memory, l4_table, raw_kernel_elf)?;

    let entry_point = kernel_elf
        .entry_point()
        .ok_or(PagingSetupError::NoEntryPoint)?;

    apply_relocations(kernel_elf)?;
    protect_relocations(kernel)?;
    unmap_module(kernel)?;

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

                if phys_addr % PAGE_SIZE != 0 {
                    panic!("Tried to map an unaligned page");
                }

                if !existing_flags.noexec() || !no_exec {
                    merged_flags = merged_flags.set_no_exec(false);
                }

                if existing_flags.writable() || writable {
                    merged_flags = merged_flags.set_writable(true);
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

                if !no_exec || !existing_flags.noexec() {
                    existing_flags = existing_flags.set_no_exec(false);
                }

                if writable || existing_flags.writable() {
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

pub fn align_up(addr: u64) -> u64 {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub fn align_down(addr: u64) -> u64 {
    addr & !(PAGE_SIZE - 1)
}
