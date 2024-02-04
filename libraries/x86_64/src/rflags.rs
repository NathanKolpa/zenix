#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct RFlags {
    value: u64,
}

impl RFlags {
    pub const NONE: RFlags = Self { value: 0 };

    pub const INTERRUPTS_ENABLED: RFlags = Self {
        value: Self::INTERRUPTS_ENABLED_BIT,
    };

    const INTERRUPTS_ENABLED_BIT: u64 = 1 << 9;

    #[cfg(target_arch = "x86_64")]
    #[doc(cfg(target_arch = "x86_64"))]
    pub fn read() -> Self {
        let value: u64;

        unsafe {
            core::arch::asm!("pushfq; pop {}", out(reg) value, options(nomem, preserves_flags));
        }

        Self { value }
    }

    pub fn as_u64(&self) -> u64 {
        self.value
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.value & Self::INTERRUPTS_ENABLED_BIT != 0
    }
}
