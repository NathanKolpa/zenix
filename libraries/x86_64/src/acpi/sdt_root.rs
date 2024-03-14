use core::{marker::PhantomData, mem::size_of};

use crate::acpi::{SDTHeader, MADT};

#[repr(packed, C)]
#[derive(Debug, Clone, Copy)]
pub struct RSDT {
    header: SDTHeader,
    array_base: PhantomData<()>,
}

impl RSDT {
    pub fn header(&self) -> &SDTHeader {
        &self.header
    }

    pub fn entries(&self) -> impl Iterator<Item = *const RSDTEntry> {
        RSDTEntryIter {
            base: (&self.array_base) as *const _ as *const u32,
            length: (self.header.length() - size_of::<SDTHeader>()) / 4,
        }
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

struct RSDTEntryIter {
    base: *const u32,
    length: usize,
}

impl Iterator for RSDTEntryIter {
    type Item = *const RSDTEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }

        let current_ptr = self.base;

        self.base = unsafe { self.base.byte_add(4) };
        self.length -= 1;

        let ptr_value = unsafe { core::ptr::read_unaligned(current_ptr) };

        Some(ptr_value as *const _)
    }
}
