mod cpu_ctx;
mod handlers;
mod int_control;
pub mod isr_wrapper;

use super::gdt::*;
pub use cpu_ctx::*;
use handlers::*;
pub use int_control::*;

use essentials::spin::Singleton;
use x86_64::interrupt::*;

pub const IRQ_START: usize = InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT;
pub const TIMER_IRQ: usize = IRQ_START;

fn init_idt() -> InterruptDescriptorTable {
    let kernel_segment = GDT.kernel_code;
    let mut idt = InterruptDescriptorTable::new();

    for isr in idt.isr_iter_mut() {
        isr.set_handler(kernel_segment, unhandled_isr)
    }

    idt.double_fault
        .set_handler(kernel_segment, double_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);

    // not setting the stack_index is important, because the page fault handler might switch
    // context. In case the kernel stack caused an overflow then a double fault will be triggered.
    idt.page_fault
        .set_handler(kernel_segment, page_fault_handler);

    idt[InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT + 0x04]
        .set_handler(kernel_segment, uart_status_change_isr);

    idt[TIMER_IRQ].set_handler(kernel_segment, tick_isr);

    idt
}

pub static IDT: Singleton<InterruptDescriptorTable> = Singleton::new(init_idt);
