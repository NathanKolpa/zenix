//! Macro's to help with debugging, aka serial printing.

#![macro_use]

use core::fmt::Write;

use x86_64::interrupt::without_interrupts;

use crate::arch::x86_64::devices::UART_16550_CHANNEL;

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::debug::_serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ($crate::debug_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _serial_print(args: core::fmt::Arguments) {
    without_interrupts(|| {
        UART_16550_CHANNEL.lock().write_fmt(args).unwrap();
    })
}

pub const DEBUG_CHANNEL: &str = "16550 UART (Serial)";
