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
