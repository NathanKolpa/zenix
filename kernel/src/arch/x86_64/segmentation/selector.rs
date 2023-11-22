use core::arch::asm;
use core::fmt::{Debug, Formatter};

use crate::arch::x86_64::PrivilegeLevel;

/// From the [osdev wiki](https://wiki.osdev.org/Segment_Selector):
/// > A Segment Selector is a 16-bit binary data structure specific to the IA-32 and x86-64 architectures.
/// > It is used in Protected Mode and Long Mode.
/// > Its value identifies a segment in either the Global Descriptor Table or a Local Descriptor Table.
/// > It contains three fields and is used in a variety of situations to interact with Segmentation.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SegmentSelector {
    value: u16,
}

impl SegmentSelector {
    pub const fn empty() -> Self {
        Self { value: 0 }
    }
    pub const fn new(index: u16, privilege: PrivilegeLevel) -> Self {
        Self {
            value: index << 3 | privilege as u16,
        }
    }

    pub const fn as_u16(&self) -> u16 {
        self.value
    }

    pub fn index(&self) -> u16 {
        self.value >> 3
    }

    pub fn privilege(&self) -> PrivilegeLevel {
        let bits = self.value & 3;
        PrivilegeLevel::from(bits as u8)
    }

    pub unsafe fn load_into_tss(&self) {
        unsafe {
            asm!("ltr {0:x}", in(reg) self.value, options(nostack, preserves_flags));
        }
    }

    pub unsafe fn load_into_cs(&self) {
        let value = self.value;
        asm!(
        "push {value}",
        "lea {tmp}, [1f + rip]",
        "push {tmp}",
        "retfq",
        "1:",
        value = in(reg) u64::from(value),
        tmp = lateout(reg) _,
        options(preserves_flags),
        );
    }

    pub unsafe fn load_into_ds(&self) {
        let _value = self.value;
        asm!("mov ds, {0:x}", in(reg) self.value, options(nostack, preserves_flags));
    }

    pub unsafe fn load_into_ss(&self) {
        let _value = self.value;
        asm!("mov ss, {0:x}", in(reg) self.value, options(nostack, preserves_flags));
    }
}

impl Debug for SegmentSelector {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SegmentSelector")
            .field("index", &self.index())
            .field("privilege", &self.privilege())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_new() {
        let selector = SegmentSelector::new(12, PrivilegeLevel::Ring2);
        assert_eq!(selector.privilege(), PrivilegeLevel::Ring2);
        assert_eq!(selector.index(), 12);
    }
}
