use bootinfo::BootInfo;
use core::{
    fmt::{Display, Formatter},
    usize,
};
use essentials::display::ReadableSize;

pub struct MemoryInfo {
    pub usable: usize,
    pub bump: usize,
    pub kernel_code: usize,
    pub kernel_heap: usize,
    pub kernel_stack: usize,
}

impl MemoryInfo {
    pub fn from_boot_info(boot_info: &BootInfo) -> Self {
        let total_allocatable_bytes = boot_info
            .usable_memory()
            .iter()
            .map(|x| x.size as usize)
            .sum();

        MemoryInfo {
            usable: total_allocatable_bytes,
            bump: boot_info.bump_memory().size as usize,
            kernel_code: boot_info.kernel_code().size as usize,
            kernel_heap: boot_info.usable_heap().size as usize,
            kernel_stack: boot_info.kernel_stack().size as usize,
        }
    }
}

impl Display for MemoryInfo {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let total = self.usable + self.bump + self.kernel_code + self.kernel_heap;

        writeln!(f, "Memory info:")?;
        writeln!(f, "\ttotal:        {}", ReadableSize::new(total))?;
        writeln!(f, "\treserved:     {}", ReadableSize::new(self.bump))?;
        writeln!(f, "\tkernel code:  {}", ReadableSize::new(self.kernel_code))?;
        writeln!(f, "\tkernel heap:  {}", ReadableSize::new(self.kernel_heap))?;
        writeln!(f, "\tkernel stack: {}", ReadableSize::new(self.kernel_stack))?;
        writeln!(f, "\tusable:       {}", ReadableSize::new(self.usable))?;
        Ok(())
    }
}
