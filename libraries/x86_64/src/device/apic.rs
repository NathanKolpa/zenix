use essentials::address::PhysicalAddress;

use crate::{rdmsr::read_apic_base, wrmsr::write_apic_base};

/// APIC ("Advanced Programmable Interrupt Controller") is the updated Intel standard for the
/// older PIC.
///
/// Resources:
///  - [Intel System Programming Guide, Vol 3A Part 1](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf).
///  - [osdev wiki](https://wiki.osdev.org/APIC).
///
/// # Mutability
///
/// From the [osdev wiki](https://wiki.osdev.org/APIC_timer):
///
/// > The great benefit of the Local APIC timer is that it is hardwired to each CPU core, unlike the Programmable Interval Timer which is a separate circuit.
/// > Because of this, there is no need for any resource management, which makes things easier.
///
/// Rust terms, this means that the Apic is automatically a thread local and can thus be be modified
/// without any locks. However, a mutable refrence is still required when the operation
/// consiststs of more than one read/write.
pub struct Apic {
    addr: PhysicalAddress,
}

impl Apic {
    pub unsafe fn from_msr() -> Self {
        let addr = read_apic_base();
        write_apic_base(addr);

        Self { addr }
    }

    unsafe fn read_register(&self, reg: usize) -> u32 {
        core::ptr::read_volatile((self.addr + reg).as_usize() as *const u32)
    }

    unsafe fn write_register(&self, reg: usize, value: u32) {
        core::ptr::write_volatile((self.addr + reg).as_usize() as *mut u32, value)
    }

    /// Set the Spurious Interrupt Vector Register bit 8 to start receiving interrupts.
    pub fn enable(&mut self) {
        unsafe {
            let register_value = self.read_register(0xF0);
            self.write_register(0xF0, register_value | 1 << 8);
        }
    }

    /// The ID of the APIC.
    ///
    /// > In MP systems, the local APIC ID is also used as a processor ID by the BIOS and the operating system.
    pub fn id(&self) -> u32 {
        unsafe { self.read_register(0x20) }
    }

    pub fn end_of_interrupt(&self) {
        unsafe { self.write_register(0xB0, 0) }
    }
}
