use core::ops::Deref;

use crate::interface::interrupts as kernel_interface;
use crate::wrap_isr;

use super::gdt::*;
use super::int_control::*;

use essentials::spin::Singleton;
use x86_64::interrupt::*;

pub const IRQ_START: usize = InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT;
pub const TIMER_IRQ: usize = IRQ_START;

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault: {frame:?}")
}

extern "x86-interrupt" fn uart_status_change_isr(_frame: InterruptStackFrame) {
    kernel_interface::uart_status_change();
}

fn tick_isr(ctx_ptr: *const InterruptedContext) -> InterruptedContext {
    let ctx = unsafe { (*ctx_ptr).clone() };
    let new_ctx = kernel_interface::tick(ctx);

    match INTERRUPT_CONTROL.deref() {
        InterruptControl::Pic(pic) => {
            without_interrupts(|| pic.lock().end_of_interrupt(TIMER_IRQ as u8));
        }
        InterruptControl::Apic(apic) => {
            apic.end_of_interrupt();
        }
    }

    new_ctx
}

wrap_isr!(tick_isr, tick_isr_handler);

extern "x86-interrupt" fn unhandled_isr(_frame: InterruptStackFrame) {
    kernel_interface::unhandled_irq()
}

fn init_idt() -> InterruptDescriptorTable {
    let kernel_segment = GDT.kernel_code;
    let mut idt = InterruptDescriptorTable::new();

    for isr in idt.isr_iter_mut() {
        isr.set_handler(kernel_segment, unhandled_isr)
    }

    idt.double_fault
        .set_handler(kernel_segment, double_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);

    idt[InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT + 0x04]
        .set_handler(kernel_segment, uart_status_change_isr);

    idt[TIMER_IRQ].set_handler(kernel_segment, tick_isr_handler);

    idt
}

pub static IDT: Singleton<InterruptDescriptorTable> = Singleton::new(init_idt);
