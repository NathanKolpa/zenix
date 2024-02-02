use crate::{memory::alloc::kernel_alloc::INITIAL_HEAP_SIZE, util::display::ReadableSize};
use bootloader_api::{info::MemoryRegionKind, BootInfo};
use core::fmt::{Display, Formatter};

pub struct MemoryInfo {
    pub usable: usize,
    pub total_size: usize,
    pub bootloader: usize,
    pub kernel_code: usize,
    pub kernel_heap: usize,
}

impl MemoryInfo {
    pub fn from_boot_info(boot_info: &BootInfo) -> Self {
        let mut total_allocatable_bytes = 0;
        let mut total_bytes = 0;
        let mut bootloader = 0;

        let regions = boot_info
            .memory_regions
            .iter()
            .map(|x| (x.kind, (x.end as usize - x.start as usize)));

        for (kind, region_size) in regions {
            match &kind {
                MemoryRegionKind::Usable => total_allocatable_bytes += region_size,
                MemoryRegionKind::Bootloader => bootloader += region_size,
                _ => {}
            }

            total_bytes += region_size;
        }

        MemoryInfo {
            usable: total_allocatable_bytes,
            total_size: total_bytes,
            bootloader,
            kernel_code: boot_info.kernel_len as usize,
            kernel_heap: INITIAL_HEAP_SIZE,
        }
    }
}

impl Display for MemoryInfo {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory info:")?;
        writeln!(f, "\ttotal:        {}", ReadableSize::new(self.total_size))?;
        writeln!(f, "\tbootloader:   {}", ReadableSize::new(self.bootloader))?;
        writeln!(f, "\tkernel code:  {}", ReadableSize::new(self.kernel_code))?;
        writeln!(f, "\tkernel heap:  {}", ReadableSize::new(self.kernel_heap))?;
        writeln!(f, "\tusable:       {}", ReadableSize::new(self.usable))?;
        Ok(())
    }
}
