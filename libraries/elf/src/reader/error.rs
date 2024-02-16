#[derive(Debug, Clone, Copy)]
pub enum ElfReadError {
    InvalidMagic,
    TooSmall,
    NotAligned,
    InvalidEntrySize,
    OverlappingEntries,
    UnsupportedBits,
    InvalidEndianness,
}

impl ElfReadError {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElfReadError::InvalidMagic => "Invlaid ELF magic",
            ElfReadError::TooSmall => "The given binary is too small",
            ElfReadError::NotAligned => "Components within the executable are not properly aligned",
            ElfReadError::InvalidEntrySize => "The entry size does not match the specification",
            ElfReadError::OverlappingEntries => "Entries overlap",
            ElfReadError::UnsupportedBits => {
                "The provided value for the bits field is not supported"
            }
            ElfReadError::InvalidEndianness => {
                "The provided value in the endianness field is not valid"
            }
        }
    }
}
