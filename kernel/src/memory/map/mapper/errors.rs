#[derive(Debug, Clone, Copy)]
pub enum NewMapError {
    NotOwned,
    OutOfFrames,
    AlreadyMapped,
}

#[derive(Debug, Clone, Copy)]
pub enum IdentityMapError {
    NotOwned,
    OutOfFrames,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum ReadMapError {
    NotMapped,
}
