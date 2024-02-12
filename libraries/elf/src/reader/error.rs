pub enum ReadError {
    InvalidMagic,
    TooSmall,
    NotAligned,
    InvalidEntrySize,
}
