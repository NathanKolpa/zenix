//! Macro's to help with debugging, aka serial printing.

#![macro_use]

mod serial_channel;

pub static CHANNEL: Singleton<SerialChannel<Uart16550>> =
    Singleton::new(|| SerialChannel::new(unsafe { Uart16550::new_and_init(0x3F8) }));

use core::fmt::Write;

use essentials::spin::Singleton;
use x86_64::device::uart_16550::Uart16550;

use crate::log::serial_channel::SerialChannel;

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::log::_serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ($crate::debug_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _serial_print(args: core::fmt::Arguments) {
    CHANNEL.writer().write_fmt(args).unwrap();
}

pub const CHANNEL_NAME: &str = "16550 UART (Serial)";
