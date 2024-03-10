#[derive(Debug, Copy, Clone)]
pub struct MemoryProperties {
    writable: bool,
    readable: bool,
    kernel: bool,
    executable: bool,
    mmio: bool,
}

impl MemoryProperties {
    pub const MMIO_PAGE: MemoryProperties = Self::new(true, true, true, false, true);
    pub const KERNEL_READ_ONLY: MemoryProperties = Self::new(false, true, true, false, false);

    pub const fn new(
        writable: bool,
        readable: bool,
        kernel: bool,
        executable: bool,
        mmio: bool,
    ) -> Self {
        Self {
            writable,
            readable,
            kernel,
            executable,
            mmio,
        }
    }

    pub const fn writable(&self) -> bool {
        self.writable
    }

    pub const fn readable(&self) -> bool {
        self.readable
    }

    pub const fn kernel(&self) -> bool {
        self.kernel
    }

    pub const fn user(&self) -> bool {
        !self.kernel()
    }

    pub const fn executable(&self) -> bool {
        self.executable
    }

    pub const fn mmio(&self) -> bool {
        self.mmio
    }
}
