pub use context::*;
pub use gate::*;
pub use idt::*;
pub use instructions::*;

use crate::arch::x86_64::segmentation::{DOUBLE_FAULT_IST_INDEX, GDT};
use crate::util::spin::Singleton;

mod context;
mod gate;
mod idt;
mod instructions;

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("Double fault: {frame:?}")
}

fn init_idt() -> InterruptDescriptorTable {
    let kernel_segment = GDT.kernel_code;
    let mut idt = InterruptDescriptorTable::new();

    idt.double_fault
        .set_handler(kernel_segment, double_fault_handler);
    idt.double_fault.set_stack_index(DOUBLE_FAULT_IST_INDEX);

    idt
}

pub static IDT: Singleton<InterruptDescriptorTable> = Singleton::new(init_idt);
