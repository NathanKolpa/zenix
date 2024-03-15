use crate::{arch::CpuContext, debug_print, memory::map::PageFault, multitasking::SCHEDULER};
use crate::{log, warning_println};

pub fn uart_status_change() {
    log::flush_availible();
}

pub fn tick(current_context: CpuContext) -> CpuContext {
    debug_print!(".");
    if let Some(next_context) = SCHEDULER.next_ctx(current_context) {
        return next_context;
    }

    todo!("Idle task is not implemented")
}

pub fn unhandled_irq() {
    warning_println!("Unhandled IRQ");
}

pub fn page_fault(fault: PageFault) -> Option<CpuContext> {
    panic!("{fault:?}");
}
