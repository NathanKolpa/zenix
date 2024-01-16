#[derive(Clone, Copy, Default, Eq, PartialEq, Debug)]
pub struct PageTableEntryFlags {
    value: u64,
}

impl PageTableEntryFlags {
    pub const fn from_page_table_entry_value(value: u64) {}

    pub const fn null() -> Self {
        Self { value: 0 }
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.value & other.value) == other.value
    }

    pub const fn set_present(&self, enabled: bool) -> Self {
        self.set_flag(0, enabled)
    }

    pub const fn set_huge(&self, enabled: bool) -> Self {
        self.set_flag(7, enabled)
    }

    pub const fn set_writable(&self, enabled: bool) -> Self {
        self.set_flag(1, enabled)
    }

    pub const fn set_user_accessible(&self, enabled: bool) -> Self {
        self.set_flag(2, enabled)
    }

    pub const fn set_no_exec(&self, enabled: bool) -> Self {
        self.set_flag(63, enabled)
    }

    pub const fn set_custom<const INDEX: u64>(&self, enabled: bool) -> Self {
        self.set_flag(Self::map_custom_bit::<INDEX>(), enabled)
    }

    const fn set_flag(&self, bit: u64, enabled: bool) -> Self {
        let value = if enabled {
            self.value | 1 << bit
        } else {
            self.value & !(1 << bit)
        };

        Self { value }
    }

    pub const fn used(&self) -> bool {
        self.value != 0
    }

    pub const fn present(&self) -> bool {
        self.value & (1 << 0) != 0
    }

    pub const fn writable(&self) -> bool {
        self.value & (1 << 1) != 0
    }

    pub const fn dirty(&self) -> bool {
        self.value & (1 << 6) != 0
    }

    pub const fn global(&self) -> bool {
        self.value & (1 << 8) != 0
    }

    pub const fn noexec(&self) -> bool {
        self.value & (1 << 63) != 0
    }

    pub const fn huge(&self) -> bool {
        self.value & (1 << 7) != 0
    }

    pub const fn user_accessible(&self) -> bool {
        self.value & (1 << 2) != 0
    }

    pub const fn custom<const INDEX: u64>(&self) -> bool {
        self.value & (1 << 9) != 0
    }

    const fn map_custom_bit<const INDEX: u64>() -> u64 {
        match INDEX {
            0..=2 => INDEX + 9,
            3..=13 => INDEX + 52 - 3,
            _ => panic!("Custom bit out of bounds"),
        }
    }

    pub const fn as_u64(&self) -> u64 {
        self.value
    }
}

impl core::ops::BitOr for PageTableEntryFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}

impl core::ops::BitAnd for PageTableEntryFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value & rhs.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_custom_bit_index() {
        assert_eq!(9, PageTableEntryFlags::map_custom_bit::<0>());
        assert_eq!(10, PageTableEntryFlags::map_custom_bit::<1>());
        assert_eq!(11, PageTableEntryFlags::map_custom_bit::<2>());
        assert_eq!(52, PageTableEntryFlags::map_custom_bit::<3>());
        assert_eq!(53, PageTableEntryFlags::map_custom_bit::<4>());
    }

    #[test_case]
    fn test_bitwise_and_eq() {
        let some_flags = PageTableEntryFlags::null()
            .set_present(true)
            .set_no_exec(true)
            .set_huge(true);

        assert_eq!(
            some_flags & PageTableEntryFlags::null(),
            PageTableEntryFlags::null()
        );

        assert_eq!(some_flags & some_flags, some_flags);
    }
}
