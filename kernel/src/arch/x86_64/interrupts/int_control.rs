use core::time::Duration;

use essentials::PanicOnce;
use x86_64::{
    cpuid::read_features,
    device::{Apic, ChainedPic8259, Pit},
    halt,
    interrupt::*,
};

use crate::arch::x86_64::{
    acpi::{AcpiInfo, ACPI_INFO},
    gdt::GDT,
    interrupts::TIMER_IRQ,
};

pub enum InterruptControl {
    Pic(ChainedPic8259),
    Apic(Apic),
}

const TIME_SLICE: Duration = Duration::from_millis(4);
const CALIBRATION_TIME: Duration = Duration::from_micros(500);

pub static INTERRUPT_CONTROL: PanicOnce<InterruptControl> = PanicOnce::new();

unsafe fn wait_pit_tick(pic: &mut ChainedPic8259, mut before: impl FnMut()) {
    static PIC: PanicOnce<&'static mut ChainedPic8259> = PanicOnce::new();

    extern "x86-interrupt" fn on_timer(_frame: InterruptStackFrame) {
        PIC.end_of_interrupt(TIMER_IRQ as u8);
    }

    let mut idt = InterruptDescriptorTable::new();
    idt[TIMER_IRQ].set_handler(GDT.kernel_code, on_timer);
    idt.load_unsafe();

    pic.allow_timer_only();
    PIC.initialize_with(&mut *(pic as *mut ChainedPic8259));

    before();
    enable_interrupts();
    let mut ticks = 0;

    while ticks <= 1 {
        halt();

        if ticks == 0 {
            before();
        }

        ticks += 1;
    }

    disable_interrupts();
}

unsafe fn init_apic(_acpi_info: &AcpiInfo, pic: &mut ChainedPic8259, pit: &mut Pit) -> Apic {
    let calibration_ratio = (TIME_SLICE.as_nanos() / CALIBRATION_TIME.as_nanos()) as u32;

    let divider = 3;

    let mut apic = Apic::from_msr();
    apic.enable(divider);

    let start_count = u32::MAX;

    pit.set_interval(CALIBRATION_TIME);
    wait_pit_tick(pic, || {
        apic.reset_counter(start_count);
    });

    let end_count = apic.stop_and_count();

    pic.allow_none();

    apic.set_periodic_mode(
        TIMER_IRQ as u32,
        (start_count - end_count) * calibration_ratio,
    );

    apic
}

/// Initialize interrupt control
///
/// # Safery
///
/// This function may unload the IDT, and assumes interrupts are disabled. Enabling interrupts
/// without loading the IDT first will cause UB.
pub unsafe fn init_interrupt_control() {
    let cpu_features = read_features();

    let mut pit = Pit::new();

    let mut pic = ChainedPic8259::new(super::IRQ_START as u8);
    pic.init();

    if let Some(info) = &*ACPI_INFO {
        if cpu_features.apic() {
            let apic = init_apic(info, &mut pic, &mut pit);

            INTERRUPT_CONTROL.initialize_with(InterruptControl::Apic(apic));
            return;
        }
    }

    pit.set_interval(TIME_SLICE);
    INTERRUPT_CONTROL.initialize_with(InterruptControl::Pic(pic));
}
