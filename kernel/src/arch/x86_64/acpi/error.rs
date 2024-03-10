use core::fmt::Debug;

use crate::memory::map::NewMapError;

#[derive(Clone, Copy)]
pub enum AcpiError {
    RsdpChecksum,
    MapRspdError(NewMapError),
}

impl Debug for AcpiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AcpiError::RsdpChecksum => write!(f, "The RSPD header checksum is incorrect"),
            AcpiError::MapRspdError(inner) => {
                write!(f, "Could not map the RSPD header ({inner:?})")
            }
        }
    }
}
