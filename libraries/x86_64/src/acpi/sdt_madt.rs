use core::{marker::PhantomData, mem::size_of};

use essentials::address::PhysicalAddress;

use crate::acpi::SDTHeader;

#[repr(C)]
#[derive(Debug)]
pub struct MADTEntry {
    kind: u8,
    length: u8,
}

#[repr(C)]
#[derive(Debug)]
pub struct MADTProcessorLocalApic {
    entry: MADTEntry,
    processor_id: u8,
    apic_id: u8,
    flags: u8,
}

impl MADTProcessorLocalApic {
    pub fn can_be_enabled(&self) -> bool {
        (self.flags & 3) != 0
    }

    pub fn processor_id(&self) -> u8 {
        self.processor_id
    }
}

#[derive(Debug)]
pub enum MADTEntryKind {
    ProcessorLocal(&'static MADTProcessorLocalApic),
    Other(&'static MADTEntry),
}

#[repr(C)]
#[derive(Debug)]
pub struct MADT {
    header: SDTHeader,
    local_apic: u32,
    flags: u32,
    array_base: PhantomData<()>,
}

impl MADT {
    pub fn header(&self) -> &SDTHeader {
        &self.header
    }

    pub fn local_apic(&self) -> PhysicalAddress {
        (self.local_apic as usize).into()
    }

    pub fn entries(&self) -> impl Iterator<Item = MADTEntryKind> + Clone {
        MADTIter {
            ptr: (&self.array_base) as *const _ as *const _,
            size: self.header.length() - size_of::<SDTHeader>(),
        }
    }
}

#[derive(Clone)]
struct MADTIter {
    ptr: *const MADTEntry,
    size: usize,
}

impl Iterator for MADTIter {
    type Item = MADTEntryKind;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size < size_of::<MADTEntry>() {
            return None;
        }

        let current_ptr = self.ptr;
        let entry_header = unsafe { &*current_ptr };

        self.size = self.size.checked_sub(entry_header.length as usize)?;
        self.ptr = unsafe { self.ptr.byte_add(entry_header.length as usize) };

        match entry_header.kind {
            0 => Some(MADTEntryKind::ProcessorLocal(unsafe {
                &*(current_ptr as *const _)
            })),
            _ => Some(MADTEntryKind::Other(entry_header)),
        }
    }
}
