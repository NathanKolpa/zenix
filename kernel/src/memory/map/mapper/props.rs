#[derive(Debug, Copy, Clone)]
pub struct MemoryProperties {
    writable: bool,
    readable: bool,
    kernel: bool,
    executable: bool,
}

impl MemoryProperties {
    pub const fn new(writable: bool, readable: bool, kernel: bool, executable: bool) -> Self {
        Self {
            writable,
            readable,
            kernel,
            executable,
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
}
