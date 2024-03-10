use core::fmt::Display;

use essentials::address::VirtualAddress;
use x86_64::acpi::RSDP;

use super::AcpiError;

pub struct AcpiInfo {
    header: &'static RSDP,
}

impl AcpiInfo {
    pub fn oem_id_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.header.oem_id).ok()
    }
}

impl Display for AcpiInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Acpi info:")?;
        writeln!(f, "\tOEM: {:?}", self.oem_id_str().unwrap_or(""))?;

        Ok(())
    }
}

pub unsafe fn parse_acpi(rsdp_addr: VirtualAddress) -> Result<AcpiInfo, AcpiError> {
    let header = &*rsdp_addr.as_ptr::<RSDP>();

    if !header.checksum_ok() {
        return Err(AcpiError::RsdpChecksum);
    }

    Ok(AcpiInfo { header })
}
