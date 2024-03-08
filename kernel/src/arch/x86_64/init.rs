use bootinfo::BootInfo;

use super::gdt::GDT;
use super::idt::IDT;
use super::int_control::init_interrupt_control;

// Initialize x86_64 specific stuff.
pub unsafe fn init(bootinfo: &BootInfo) {
    init_interrupt_control(bootinfo);

    GDT.load();
    IDT.load();
}
