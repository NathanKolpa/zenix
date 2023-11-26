use crate::util::display::ReadableSize;
use core::fmt::{Display, Formatter};

pub struct MemoryInfo {
    pub usable: usize,
    pub allocated: usize,
    pub total_size: usize,
    pub kernel: usize,
}

impl Display for MemoryInfo {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Memory snapshot:")?;
        writeln!(f, "\ttotal:       {}", ReadableSize::new(self.total_size))?;
        writeln!(f, "\tkernel code: {}", ReadableSize::new(self.kernel))?;
        writeln!(f, "\tusable:      {}", ReadableSize::new(self.usable))?;
        writeln!(f, "\tallocated:   {}", ReadableSize::new(self.allocated))?;
        write!(f,   "\tfree:        {}", ReadableSize::new(self.usable - self.allocated))?;
        Ok(())
    }
}
