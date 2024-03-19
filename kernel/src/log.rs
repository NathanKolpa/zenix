//! Macro's to help with debugging, aka serial printing.

#![macro_use]

mod buffer_logger;
mod log_mux;
mod macros;
mod serial_logger;

use core::fmt::Arguments;

use essentials::spin::Singleton;
use x86_64::device::{Uart16550, VgaBuffer};

use crate::log::{buffer_logger::BufferLogger, log_mux::LoggerMux, serial_logger::SerialLogger};

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

static CHANNEL: Singleton<LoggerMux<SerialLogger<Uart16550>, BufferLogger<VgaBuffer>>> =
    Singleton::new(|| unsafe {
        LoggerMux::new(
            SerialLogger::new(Uart16550::new_and_init(0x3F8)),
            BufferLogger::new(VgaBuffer::new()),
        )
    });

pub fn flush_availible() {
    CHANNEL.flush();
}

#[doc(hidden)]
pub fn _channel_print(level: LogLevel, args: core::fmt::Arguments) {
    CHANNEL.log(level, args);
}

pub const CHANNEL_NAME: &str = "16550 UART (Serial) + VGA (Text buffer)";
