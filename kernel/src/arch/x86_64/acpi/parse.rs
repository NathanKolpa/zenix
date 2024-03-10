use core::{fmt::Display, mem::size_of};

use essentials::address::{PhysicalAddress, VirtualAddress};
use x86_64::acpi::{RSDTEntry, RSDTEntryKind, RSDP, RSDT};

use crate::{
    memory::map::{MemoryMapper, MemoryProperties},
    warning_println,
};

use super::AcpiError;

pub struct AcpiInfo {
    oem_id: Option<&'static str>,
    local_apic_ptr: Option<PhysicalAddress>,
}

impl Display for AcpiInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Acpi info:")?;
        writeln!(f, "\tOEM       : {:?}", self.oem_id.unwrap_or(""))?;
        writeln!(f, "\tLocal APIC: {:?}", self.local_apic_ptr)?;

        Ok(())
    }
}

pub unsafe fn parse_acpi(
    rsdp_addr: VirtualAddress,
    mapper: &mut MemoryMapper,
) -> Result<AcpiInfo, AcpiError> {
    mapper
        .identity_map(
            PhysicalAddress::new(rsdp_addr.as_usize()),
            size_of::<RSDP>(),
            MemoryProperties::KERNEL_READ_ONLY,
        )
        .map_err(AcpiError::MapRspdError)?;

    let header = &*rsdp_addr.as_ptr::<RSDP>();

    if !header.checksum_ok() {
        return Err(AcpiError::RsdpChecksum);
    }

    if header.extended() {
        warning_println!("Not making use of extended ACPI tables");
    }

    mapper
        .identity_map(
            header.rsdt_addr(),
            size_of::<RSDT>(),
            MemoryProperties::KERNEL_READ_ONLY,
        )
        .map_err(AcpiError::MapRsdtError)?;

    let sdt_root_ptr = header.rsdt_addr().as_usize() as *const RSDT;
    let sdt_root = &*sdt_root_ptr;

    if !sdt_root.header().checksum_ok() {
        return Err(AcpiError::RsdtChecksum);
    }

    let mut local_apic_ptr = None;

    for entry_ptr in sdt_root.iter() {
        mapper
            .identity_map(
                PhysicalAddress::from(entry_ptr),
                size_of::<RSDTEntry>(),
                MemoryProperties::KERNEL_READ_ONLY,
            )
            .map_err(AcpiError::MapEntryError)?;

        let entry = &*entry_ptr;

        if !entry.header().checksum_ok() {
            return Err(AcpiError::EntryChecksum(entry.header().signature()));
        }

        match entry.kind() {
            RSDTEntryKind::Madt(madt) => local_apic_ptr = Some(madt.local_apic()),
            RSDTEntryKind::Other(_) => {}
        }
    }

    Ok(AcpiInfo {
        oem_id: header.oem_id(),
        local_apic_ptr,
    })
}
