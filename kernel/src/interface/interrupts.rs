use crate::{arch::CpuContext, multitasking::SCHEDULER};
use crate::{log, warning_println};

pub fn uart_status_change() {
    log::CHANNEL.flush_availible();
}

pub fn tick(current_context: CpuContext) -> CpuContext {
    if let Some(next_context) = SCHEDULER.next_ctx(current_context) {
        return next_context;
    }

    todo!("Idle task is not implemented")
}

pub fn unhandled_irq() {
    warning_println!("Unhandled IRQ");
}
