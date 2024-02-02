use crate::util::display::ReadableSize;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use core::fmt::{Display, Formatter};

pub struct MemoryInfo {
    pub usable: usize,
    pub total_size: usize,
    pub bootloader: usize,
}

impl MemoryInfo {
    pub fn from_memory_map(memory_map: &MemoryRegions) -> Self {
        let mut total_allocatable_bytes = 0;
        let mut total_bytes = 0;
        let mut bootloader = 0;

        let regions = memory_map
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
        }
    }
}

impl Display for MemoryInfo {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory info:")?;
        writeln!(f, "\ttotal:        {}", ReadableSize::new(self.total_size))?;
        writeln!(f, "\tbootloader:   {}", ReadableSize::new(self.bootloader))?;
        writeln!(f, "\tusable:       {}", ReadableSize::new(self.usable))?;
        Ok(())
    }
}
