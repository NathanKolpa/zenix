use essentials::address::PhysicalAddress;

use crate::acpi::SDTHeader;

#[repr(C)]
#[derive(Debug)]
pub struct MADT {
    header: SDTHeader,
    local_apic: u32,
    flags: u32,
}

impl MADT {
    pub fn header(&self) -> &SDTHeader {
        &self.header
    }

    pub fn local_apic(&self) -> PhysicalAddress {
        (self.local_apic as usize).into()
    }
}
