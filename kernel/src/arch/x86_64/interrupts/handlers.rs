use super::{InterruptControl, INTERRUPT_CONTROL};
use crate::{
    interface::interrupts as kernel_interface,
    memory::map::manager::{MemoryAccess, MemoryViolation, PageFault},
};
use x86_64::{
    interrupt::{
        InterruptErrorContext, InterruptStackFrame, InterruptedContext, PageFaultErrorCode,
    },
    paging::cr2,
};

pub extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Double fault ({frame:?})")
}

pub fn page_fault(ctx: &InterruptErrorContext<PageFaultErrorCode>) -> Option<InterruptedContext> {
    let addr = cr2::page_fault_addr();

    let access = if ctx.error.instruction_fetch() {
        MemoryAccess::InstructionFetch
    } else if ctx.error.caused_by_write() {
        MemoryAccess::Write
    } else {
        MemoryAccess::Read
    };

    let violation = if ctx.error.malformed_table() {
        MemoryViolation::MalformedTable
    } else if ctx.error.protection_violation() || ctx.error.protection_key() {
        MemoryViolation::InsufficientPrivilge
    } else {
        MemoryViolation::NotMapped
    };

    let fault = PageFault {
        instruction_pointer: ctx.context.interrupt_stack_frame().instruction_pointer,
        addr,
        access,
        violation,
    };

    kernel_interface::page_fault(fault)
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
            pic.end_of_interrupt(super::TIMER_IRQ as u8);
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
crate::wrap_error_isr!(page_fault, page_fault_handler, PageFaultErrorCode);
