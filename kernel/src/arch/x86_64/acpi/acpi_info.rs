use core::fmt::Display;

pub struct AcpiInfo {
    pub oem_id: [u8; 6],
}

impl AcpiInfo {
    pub fn oem_id_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.oem_id).ok()
    }
}

impl Display for AcpiInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Acpi info:")?;
        writeln!(f, "\tOEM: {:?}", self.oem_id_str().unwrap_or(""))?;

        Ok(())
    }
}
