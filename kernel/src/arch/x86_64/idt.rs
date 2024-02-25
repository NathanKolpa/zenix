use crate::interface::interrupts;

use super::gdt::*;
use essentials::spin::{Singleton, SpinLock};
use x86_64::{device::pic_8259::ChainedPic8259, interrupt::*};

const PIC_CHAIN_INTS_START: usize = InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT;
const PIC_CHAIN_TICK_INT_INDEX: usize = PIC_CHAIN_INTS_START;

pub static PIC_CHAIN: Singleton<SpinLock<ChainedPic8259>> =
    Singleton::new(|| SpinLock::new(unsafe { ChainedPic8259::new(PIC_CHAIN_INTS_START as u8) }));

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault: {frame:?}")
}

extern "x86-interrupt" fn uart_status_change_isr(_frame: InterruptStackFrame) {
    interrupts::uart_status_change();
}

extern "x86-interrupt" fn tick_isr(_frame: InterruptStackFrame) {
    PIC_CHAIN
        .lock()
        .end_of_interrupt(PIC_CHAIN_TICK_INT_INDEX as u8);
}

fn init_idt() -> InterruptDescriptorTable {
    let kernel_segment = GDT.kernel_code;
    let mut idt = InterruptDescriptorTable::new();

    idt.double_fault
        .set_handler(kernel_segment, double_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);

    idt[InterruptDescriptorTable::STANDARD_INTERRUPTS_COUNT + 0x04]
        .set_handler(kernel_segment, uart_status_change_isr);

    idt[PIC_CHAIN_TICK_INT_INDEX].set_handler(kernel_segment, tick_isr);

    idt
}

pub static IDT: Singleton<InterruptDescriptorTable> = Singleton::new(init_idt);
