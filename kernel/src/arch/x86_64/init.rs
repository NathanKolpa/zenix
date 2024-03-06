use super::gdt::GDT;
use super::idt::IDT;

// Initialize x86_64 specific stuff.
pub fn init() {
    GDT.load();
    IDT.load();
}
