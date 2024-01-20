pub enum NewMapError {
    NotOwned,
    OutOfFrames,
    AlreadyMapped,
}

#[derive(Debug)]
pub enum ModifyMapError {
    NotOwned,
    NotMapped,
}

pub enum ReadMapError {
    NotMapped,
    InconsistencyWithinRange,
}
