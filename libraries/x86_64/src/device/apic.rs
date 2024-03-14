use essentials::address::PhysicalAddress;

#[repr(usize)]
enum Reg {
    Id = 0x20,
    Spurious = 0x0F0,
    Divider = 0x3E0,
    EndOfInt = 0x0B0,
    LvtTimer = 0x320,
    InitialCount = 0x380,
    CurrentCount = 0x390,
    Priority = 0x80,
}

/// APIC ("Advanced Programmable Interrupt Controller") is the updated Intel standard for the
/// older PIC.
///
/// Resources:
///  - [Intel System Programming Guide, Vol 3A Part 1](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf).
///  - [osdev wiki](https://wiki.osdev.org/APIC).
pub struct Apic {
    addr: PhysicalAddress,
}

impl Apic {
    #[cfg(target_arch = "x86_64")]
    #[doc(cfg(target_arch = "x86_64"))]
    pub unsafe fn from_msr() -> Self {
        let addr = crate::rdmsr::read_apic_base();
        crate::wrmsr::write_apic_base(addr);

        Self { addr }
    }

    unsafe fn read_register(&self, reg: Reg) -> u32 {
        core::ptr::read_volatile((self.addr + reg as usize).as_usize() as *const u32)
    }

    unsafe fn write_register(&self, reg: Reg, value: u32) {
        core::ptr::write_volatile((self.addr + reg as usize).as_usize() as *mut u32, value)
    }

    pub fn enable(&mut self, divider: u32) {
        unsafe {
            self.write_register(Reg::Priority, 0);
            self.write_register(Reg::Divider, divider);

            let value = self.read_register(Reg::Spurious);
            self.write_register(Reg::Spurious, value | 1 << 8);
        }
    }

    pub fn set_periodic_mode(&mut self, irq: u32, initial_count: u32) {
        const PERIODIC_MODE: u32 = 0x20000;

        unsafe {
            self.write_register(Reg::LvtTimer, irq | PERIODIC_MODE);
            self.write_register(Reg::InitialCount, initial_count);
        }
    }

    pub fn stop_and_count(&mut self) -> u32 {
        const DISABLE: u32 = 0x10000;
        unsafe {
            self.write_register(Reg::LvtTimer, DISABLE);
            self.read_register(Reg::CurrentCount)
        }
    }

    pub fn reset_counter(&mut self, initial_count: u32) {
        unsafe {
            self.write_register(Reg::InitialCount, initial_count);
        }
    }

    /// The ID of the APIC.
    ///
    /// > In MP systems, the local APIC ID is also used as a processor ID by the BIOS and the operating system.
    pub fn id(&self) -> u32 {
        unsafe { self.read_register(Reg::Id) }
    }

    pub fn end_of_interrupt(&self) {
        unsafe { self.write_register(Reg::EndOfInt, 0) }
    }
}
