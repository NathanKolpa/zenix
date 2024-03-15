use super::Logger;

pub struct LoggerMux<A, B> {
    a: A,
    b: B,
}

impl<A, B> LoggerMux<A, B>
where
    A: Logger,
    B: Logger,
{
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Logger for LoggerMux<A, B>
where
    A: Logger,
    B: Logger,
{
    fn log(&self, level: super::LogLevel, args: core::fmt::Arguments<'_>) {
        self.a.log(level, args);
        self.b.log(level, args);
    }

    fn flush(&self) {
        self.a.flush();
        self.b.flush();
    }
}
