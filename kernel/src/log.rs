//! Macro's to help with debugging, aka serial printing.

#![macro_use]

mod buffer_logger;
mod log_mux;
mod serial_logger;

use core::fmt::Arguments;

use essentials::spin::Singleton;
use x86_64::device::{Uart16550, VgaBuffer};

use crate::log::{buffer_logger::BufferLogger, log_mux::SerialMux, serial_logger::SerialLogger};

/// Information that is diagnostically helpful to people more than just developers.
#[macro_export]
macro_rules! info_print {
    ($($arg:tt)*) => ($crate::log::_channel_print($crate::log::LogLevel::Info, format_args!($($arg)*)));
}

/// Information that is diagnostically helpful to people more than just developers.
#[macro_export]
macro_rules! info_println {
    () => ($crate::info_print!("\n"));
    ($($arg:tt)*) => ($crate::info_print!("{}\n", format_args!($($arg)*)));
}

/// Anything that can potentially cause application oddities, but for which I am automatically recovering.
#[macro_export]
macro_rules! warning_print {
    ($($arg:tt)*) => ($crate::log::_channel_print($crate::log::LogLevel::Warn, format_args!($($arg)*)));
}

/// Anything that can potentially cause application oddities, but for which I am automatically recovering.
#[macro_export]
macro_rules! warning_println {
    () => ($crate::warning_print!("\n"));
    ($($arg:tt)*) => ($crate::warning_print!("{}\n", format_args!($($arg)*)));
}

/// Used for errors that force the kernel to shutdown.
#[macro_export]
macro_rules! error_print {
    ($($arg:tt)*) => ($crate::log::_channel_print($crate::log::LogLevel::Error, format_args!($($arg)*)));
}

/// Used for errors that force the kernel to shutdown.
#[macro_export]
macro_rules! error_println {
    () => ($crate::error_print!("\n"));
    ($($arg:tt)*) => ($crate::error_print!("{}\n", format_args!($($arg)*)));
}

/// Used for debugging the kernel and development tasks.
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => ($crate::log::_channel_print($crate::log::LogLevel::Debug, format_args!($($arg)*)));
}

/// Used for debugging the kernel and development tasks.
#[macro_export]
macro_rules! debug_println {
    () => ($crate::debug_print!("\n"));
    ($($arg:tt)*) => ($crate::debug_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Warn,
    Error,
    Info,
}

trait Logger {
    fn log(&self, level: LogLevel, args: Arguments<'_>);
    fn flush(&self);
}

pub fn flush_availible() {
    CHANNEL.flush();
}

static CHANNEL: Singleton<SerialMux<SerialLogger<Uart16550>, BufferLogger<VgaBuffer>>> =
    Singleton::new(|| unsafe {
        SerialMux::new(
            SerialLogger::new(Uart16550::new_and_init(0x3F8)),
            BufferLogger::new(VgaBuffer::new()),
        )
    });

#[doc(hidden)]
pub fn _channel_print(level: LogLevel, args: core::fmt::Arguments) {
    CHANNEL.log(level, args);
}

pub const CHANNEL_NAME: &str = "16550 UART (Serial) + VGA (Text buffer)";
