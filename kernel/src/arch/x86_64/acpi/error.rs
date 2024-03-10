use core::fmt::Debug;

use crate::memory::map::NewMapError;

#[derive(Clone, Copy)]
pub enum AcpiError {
    RsdpChecksum,
    RsdtChecksum,
    EntryChecksum(Option<&'static str>),
    MapRspdError(NewMapError),
    MapRsdtError(NewMapError),
    MapEntryError(NewMapError),
}

impl Debug for AcpiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AcpiError::RsdpChecksum => write!(f, "The RSPD header checksum is incorrect"),
            AcpiError::RsdtChecksum => write!(f, "The RSDT checksum is incorrect"),
            AcpiError::MapRspdError(inner) => {
                write!(f, "Could not map the RSPD header ({inner:?})")
            }
            AcpiError::MapRsdtError(inner) => {
                write!(
                    f,
                    "Could not map the Root system description table ({inner:?})"
                )
            }
            AcpiError::EntryChecksum(sig) => {
                write!(
                    f,
                    "System Descriptor table checksum with signature {sig:?} is incorrect"
                )

            },
            AcpiError::MapEntryError(inner) =>{ 
                write!(
                    f,
                    "Could not map the Root system description table ({inner:?})"
                )

            }
        }
    }
}
