use super::{InterruptControl, INTERRUPT_CONTROL};
use crate::interface::interrupts as kernel_interface;
use x86_64::interrupt::{InterruptStackFrame, InterruptedContext};

pub extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Double fault: {frame:?}")
}

fn uart_status_change(_ctx: &InterruptedContext) -> Option<InterruptedContext> {
    kernel_interface::uart_status_change();

    None
}

fn unhandled(_ctx: &InterruptedContext) -> Option<InterruptedContext> {
    kernel_interface::unhandled_irq();

    None
}

fn tick(ctx: &InterruptedContext) -> Option<InterruptedContext> {
    let new_ctx = kernel_interface::tick(ctx.clone());

    match &*INTERRUPT_CONTROL {
        InterruptControl::Pic(pic) => {
            pic.guard().lock().end_of_interrupt(super::TIMER_IRQ as u8);
        }
        InterruptControl::Apic(apic) => {
            apic.end_of_interrupt();
        }
    }

    Some(new_ctx)
}

crate::wrap_isr!(uart_status_change, uart_status_change_isr);
crate::wrap_isr!(unhandled, unhandled_isr);
crate::wrap_isr!(tick, tick_isr);
