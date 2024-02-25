use crate::log;

pub fn uart_status_change() {
    log::CHANNEL.flush_availible();
}
