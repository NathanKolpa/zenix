use crate::util::display::ReadableSize;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::fmt::{Display, Formatter};

pub struct MemoryInfo {
    pub usable: usize,
    pub total_size: usize,
    pub kernel: usize,
}

impl MemoryInfo {
    pub fn from_memory_map(memory_map: &MemoryMap) -> Self {
        let mut total_allocatable_bytes = 0;
        let mut total_bytes = 0;
        let mut kernel = 0;

        let regions = memory_map.iter().map(|x| {
            (
                x.region_type,
                (x.range.end_frame_number as usize * 4096
                    - x.range.start_frame_number as usize * 4096),
            )
        });

        for (kind, region_size) in regions {
            match &kind {
                MemoryRegionType::Usable => total_allocatable_bytes += region_size,
                MemoryRegionType::KernelStack | MemoryRegionType::Kernel => kernel += region_size,
                _ => {}
            }

            if kind != MemoryRegionType::Reserved {
                total_bytes += region_size;
            }
        }

        MemoryInfo {
            usable: total_allocatable_bytes,
            total_size: total_bytes,
            kernel,
        }
    }
}

impl Display for MemoryInfo {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory info:")?;
        writeln!(f, "\ttotal:       {}", ReadableSize::new(self.total_size))?;
        writeln!(f, "\tkernel code: {}", ReadableSize::new(self.kernel))?;
        writeln!(f, "\tusable:      {}", ReadableSize::new(self.usable))?;
        Ok(())
    }
}
