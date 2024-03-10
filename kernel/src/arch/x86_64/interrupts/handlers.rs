use super::{InterruptControl, INTERRUPT_CONTROL};
use crate::interface::interrupts as kernel_interface;
use x86_64::interrupt::{InterruptStackFrame, InterruptedContext};

pub extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Double fault: {frame:?}")
}

pub extern "x86-interrupt" fn uart_status_change_isr(_frame: InterruptStackFrame) {
    kernel_interface::uart_status_change();
}

pub extern "x86-interrupt" fn unhandled_isr(_frame: InterruptStackFrame) {
    kernel_interface::unhandled_irq()
}

fn tick_isr_inner(ctx_ptr: *const InterruptedContext) -> InterruptedContext {
    let ctx = unsafe { (*ctx_ptr).clone() };
    let new_ctx = kernel_interface::tick(ctx);

    match &*INTERRUPT_CONTROL {
        InterruptControl::Pic(pic) => {
            pic.guard().lock().end_of_interrupt(super::TIMER_IRQ as u8);
        }
        InterruptControl::Apic(apic) => {
            apic.end_of_interrupt();
        }
    }

    new_ctx
}

crate::wrap_isr!(tick_isr_inner, tick_isr);
