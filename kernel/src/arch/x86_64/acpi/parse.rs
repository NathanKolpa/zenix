use core::{fmt::Display, mem::size_of};

use alloc::{boxed::Box, vec::Vec};
use essentials::address::{PhysicalAddress, VirtualAddress};
use x86_64::acpi::*;

use crate::{
    memory::map::{MemoryMapper, MemoryProperties},
    warning_println,
};

use super::AcpiError;

pub struct AcpiProcessor {
    id: u8,
}

pub struct AcpiInfo {
    oem_id: Option<&'static str>,
    local_apic_ptr: Option<PhysicalAddress>,
    processors: Box<[AcpiProcessor]>,
}

impl AcpiInfo {
    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }
}

impl Display for AcpiInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Acpi info:")?;

        writeln!(f, "\tProcessor count: {:?}", self.processor_count())?;

        if let Some(oem) = self.oem_id {
            writeln!(f, "\tOEM:             {oem}")?;
        }

        if let Some(apic) = self.local_apic_ptr {
            writeln!(f, "\tLocal APIC:      {apic}")?;
        }

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
    let mut processors = Vec::with_capacity(64);

    for entry_ptr in sdt_root.entries() {
        // Its not garanteed that the entry is mapped.
        mapper
            .identity_map(
                PhysicalAddress::from(entry_ptr),
                size_of::<RSDTEntry>(),
                MemoryProperties::KERNEL_READ_ONLY,
            )
            .map_err(AcpiError::MapEntryError)?;

        let entry = &*entry_ptr;

        // The length was unknown untill now. So identity map has to be called agian.
        mapper
            .identity_map(
                PhysicalAddress::from(entry_ptr),
                entry.header().length(),
                MemoryProperties::KERNEL_READ_ONLY,
            )
            .map_err(AcpiError::MapEntryError)?;

        if !entry.header().checksum_ok() {
            return Err(AcpiError::EntryChecksum(entry.header().signature()));
        }

        match entry.kind() {
            RSDTEntryKind::Madt(madt) => {
                parse_madt(madt, &mut processors, &mut local_apic_ptr)?;
            }
            RSDTEntryKind::Other(_) => {}
        }
    }

    Ok(AcpiInfo {
        oem_id: header.oem_id(),
        local_apic_ptr,
        processors: processors.into_boxed_slice(),
    })
}

unsafe fn parse_madt(
    madt: &MADT,
    processors: &mut Vec<AcpiProcessor>,
    local_apic_ptr: &mut Option<PhysicalAddress>,
) -> Result<(), AcpiError> {
    for madt_entry in madt.entries() {
        match madt_entry {
            MADTEntryKind::ProcessorLocal(proc_local) => {
                if !proc_local.can_be_enabled() {
                    continue;
                }

                processors.push(AcpiProcessor {
                    id: proc_local.processor_id(),
                });
            }
            MADTEntryKind::Other(_) => {}
        }
    }

    *local_apic_ptr = Some(madt.local_apic());
    Ok(())
}
