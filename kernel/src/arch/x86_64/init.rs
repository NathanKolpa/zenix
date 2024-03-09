use bootinfo::BootInfo;

use crate::{info_print, warning_println};

use super::acpi::ACPI_INFO;
use super::idt::IDT;
use super::int_control::init_interrupt_control;
use super::{acpi::init_acpi, gdt::GDT};

// Initialize x86_64 specific stuff.
pub unsafe fn init(bootinfo: &BootInfo) {
    if let Err(acpi_err) = init_acpi(bootinfo) {
        warning_println!("ACPI Error: {acpi_err:?}");
        warning_println!("Not all hardware features will be supported");
    }

    if let Some(acpi_info) = &*ACPI_INFO {
        info_print!("{acpi_info}");
    }

    init_interrupt_control();

    GDT.load();
    IDT.load();
}
