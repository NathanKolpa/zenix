use core::fmt::Debug;

#[derive(Clone, Copy)]
pub enum AcpiError {
    RsdpChecksum,
}

impl Debug for AcpiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AcpiError::RsdpChecksum => f.write_str("The RSPD header checksum is incorrect"),
        }
    }
}
