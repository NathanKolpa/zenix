//! Macro's to help with debugging, aka serial printing.

#![macro_use]

mod serial_channel;

pub static CHANNEL: Singleton<SerialChannel<Uart16550>> =
    Singleton::new(|| SerialChannel::new(unsafe { Uart16550::new_and_init(0x3F8) }));

use core::fmt::Write;

use essentials::spin::Singleton;
use x86_64::device::uart_16550::Uart16550;

use crate::log::serial_channel::SerialChannel;

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
pub enum LogLevel {
    Debug,
    Warn,
    Error,
    Info,
}

impl LogLevel {
    pub fn ansi_color_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "\x1b[90m",
            LogLevel::Warn => "\x1b[33m",
            LogLevel::Error => "\x1b[31m",
            LogLevel::Info => "\x1b[36m",
        }
    }
}

#[doc(hidden)]
pub fn _channel_print(level: LogLevel, args: core::fmt::Arguments) {
    let color_str = level.ansi_color_str();
    let reset_str = "\x1b[0m";

    let mut writer = CHANNEL.writer();

    writer.write_str(color_str).unwrap();
    writer.write_fmt(args).unwrap();
    writer.write_str(reset_str).unwrap();
}

pub const CHANNEL_NAME: &str = "16550 UART (Serial)";
