use bootinfo::BootInfo;
use x86_64::cpuid;

use crate::{info_print, info_println, memory::map::MemoryMapper, warning_println};

use super::acpi::ACPI_INFO;
use super::interrupts::init_interrupt_control;
use super::interrupts::IDT;
use super::{acpi::init_acpi, gdt::GDT};

// Initialize x86_64 specific stuff.
pub unsafe fn init(bootinfo: &BootInfo, mapper: &mut MemoryMapper) {
    if let Err(acpi_err) = init_acpi(bootinfo, mapper) {
        warning_println!("ACPI Error: {acpi_err:?}");
        warning_println!("Not all hardware features will be supported");
    }

    if let Some(acpi_info) = &*ACPI_INFO {
        info_print!("{acpi_info}");
    }

    let features = cpuid::read_features();
    info_println!("CPUID features: {features}");

    init_interrupt_control();

    GDT.load();
    IDT.load();
}
