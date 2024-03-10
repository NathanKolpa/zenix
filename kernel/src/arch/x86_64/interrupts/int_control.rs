use essentials::{spin::SpinLock, PanicOnce};
use x86_64::{
    cpuid::read_features,
    device::{apic::Apic, pic_8259::ChainedPic8259},
};

use crate::{arch::x86_64::acpi::{AcpiInfo, ACPI_INFO}, utils::InterruptGuard};


pub enum InterruptControl {
    Pic(InterruptGuard<SpinLock<ChainedPic8259>>),
    Apic(Apic),
}

pub static INTERRUPT_CONTROL: PanicOnce<InterruptControl> = PanicOnce::new();

unsafe fn init_apic(_acpi_info: &AcpiInfo, pic: &mut ChainedPic8259) -> Apic {
    pic.disable();

    let mut apic = Apic::from_msr();
    apic.enable();

    apic
}

pub unsafe fn init_interrupt_control() {
    let cpu_features = read_features();

    let mut pic = ChainedPic8259::new(super::IRQ_START as u8);
    pic.init();

    if let Some(info) = &*ACPI_INFO {
        if cpu_features.apic() {
            let apic = init_apic(info, &mut pic);

            INTERRUPT_CONTROL.initialize_with(InterruptControl::Apic(apic));
            return;
        }
    }

    INTERRUPT_CONTROL.initialize_with(InterruptControl::Pic(InterruptGuard::new_lock(pic)));
}
