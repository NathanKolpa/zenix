pub enum NewMapError {
    NotOwned,
    OutOfFrames,
    AlreadyMapped,
}

pub enum ModifyMapError {
    NotOwned,
    NotMapped,
}

pub enum ReadMapError {
    NotMapped,
    InconsistentRange,
}
