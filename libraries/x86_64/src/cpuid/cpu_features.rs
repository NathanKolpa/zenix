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
