use core::mem::size_of;

use crate::acpi::{SDTHeader, MADT};

#[repr(C)]
#[derive(Debug)]
pub struct RSDT {
    header: SDTHeader,
    array_base: u32,
}

impl RSDT {
    pub fn header(&self) -> &SDTHeader {
        &self.header
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = *const RSDTEntry> {
        let ptr_array = unsafe {
            core::slice::from_raw_parts(
                (&self.array_base) as *const u32,
                (self.header.length() - size_of::<SDTHeader>()) / 4,
            )
        };

        ptr_array.iter().copied().map(|ptr| ptr as *const RSDTEntry)
    }
}

#[derive(Debug)]
pub enum RSDTEntryKind {
    Madt(&'static MADT),
    Other(&'static SDTHeader),
}

#[derive(Debug)]
#[repr(C)]
pub struct RSDTEntry {
    header: SDTHeader,
}

impl RSDTEntry {
    pub fn header(&self) -> &SDTHeader {
        &self.header
    }

    pub fn kind(&'static self) -> RSDTEntryKind {
        match self.header().signature() {
            Some("APIC") => RSDTEntryKind::Madt(unsafe { &*(self as *const _ as *const MADT) }),
            _ => RSDTEntryKind::Other(self.header()),
        }
    }
}
