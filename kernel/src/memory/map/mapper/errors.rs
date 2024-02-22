#[derive(Debug)]
pub enum NewMapError {
    NotOwned,
    OutOfFrames,
    AlreadyMapped,
}

#[derive(Debug)]
pub enum ModifyMapError {
    NotOwned,
    NotMapped,
    OutOfBounds,
}

impl From<ModifyMapError> for NewMapError {
    fn from(value: ModifyMapError) -> Self {
        match value {
            ModifyMapError::NotOwned => NewMapError::NotOwned,
            ModifyMapError::NotMapped => unreachable!(),
            ModifyMapError::OutOfBounds => unreachable!(),
        }
    }
}

pub enum ReadMapError {
    NotMapped,
    InconsistencyWithinRange,
}
