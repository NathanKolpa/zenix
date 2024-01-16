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
}
