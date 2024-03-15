use core::fmt::Display;

use essentials::display::Quoteless;

pub struct CpuFeatures {
    ecx: u64,
    edx: u64,
}

impl CpuFeatures {
    pub const fn new(ecx: u64, edx: u64) -> Self {
        Self { ecx, edx }
    }

    pub const fn apic(&self) -> bool {
        self.edx & (1 << 9) != 0
    }

    pub const fn sse3(&self) -> bool {
        self.ecx & (1 << 0) != 0
    }
}

impl Display for CpuFeatures {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut set = f.debug_set();

        if self.sse3() {
            set.entry(&Quoteless::new("SSE3"));
        }

        if self.apic() {
            set.entry(&Quoteless::new("APIC"));
        }

        set.finish()
    }
}
