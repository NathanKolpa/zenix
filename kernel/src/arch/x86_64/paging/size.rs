const FOUR_KIB: usize = 4096;
pub const TABLE_ENTRIES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageSize {
    Size4Kib,
    Size2Mib,
    Size1Gib,
}

impl PageSize {
    pub const HIGHEST: Self = PageSize::Size1Gib;

    pub const fn descend_level(&self) -> Self {
        match self {
            PageSize::Size4Kib => PageSize::Size4Kib,
            PageSize::Size2Mib => PageSize::Size4Kib,
            PageSize::Size1Gib => PageSize::Size2Mib,
        }
    }

    pub const fn as_usize(&self) -> usize {
        match self {
            PageSize::Size4Kib => FOUR_KIB,
            PageSize::Size2Mib => FOUR_KIB * TABLE_ENTRIES,
            PageSize::Size1Gib => FOUR_KIB * TABLE_ENTRIES * TABLE_ENTRIES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_as_usize() {
        assert_eq!(4096, PageSize::Size4Kib.as_usize());
        assert_eq!(2097152, PageSize::Size2Mib.as_usize());
        assert_eq!(1073741824, PageSize::Size1Gib.as_usize());
    }

    #[test_case]
    fn test_descend_level() {
        assert_eq!(PageSize::Size4Kib, PageSize::Size4Kib.descend_level());
        assert_eq!(PageSize::Size4Kib, PageSize::Size2Mib.descend_level());
        assert_eq!(PageSize::Size2Mib, PageSize::Size1Gib.descend_level());
    }
}
