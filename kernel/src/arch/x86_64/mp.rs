use super::acpi::ACPI_INFO;
use super::interrupts::{InterruptControl, INTERRUPT_CONTROL};

pub fn processor_count() -> usize {
    match &*ACPI_INFO {
        Some(info) => info.processor_count(),
        None => 1,
    }
}

pub fn processor_id() -> usize {
    match &*INTERRUPT_CONTROL {
        InterruptControl::Pic(_) => 0,
        InterruptControl::Apic(apic) => apic.id() as usize,
    }
}

pub fn start_slave_processors(_startup: impl Fn(usize)) {}
