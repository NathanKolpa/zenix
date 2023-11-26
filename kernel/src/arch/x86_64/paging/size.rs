const FOUR_KIB: usize = 4096;
const TABLE_ENTRIES: usize = 512;

#[derive(Clone, Copy, Debug)]
pub enum PageSize {
    Size4Kib,
    Size2Mib,
    Size1Gib,
}

impl PageSize {
    pub const fn from_level(level: u8) -> Self {
        match level {
            2 => PageSize::Size2Mib,
            3 => PageSize::Size1Gib,
            1 | 4 => PageSize::Size4Kib,
            _ => panic!("Page level must be between 1 and 4"),
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
