use core::fmt::Write;

use essentials::spin::SpinLock;
use x86_64::device::{ColouredTextBufferReader, ColouredTextBufferWriter, FrameBuffer, TextColour};

const TAB_WIDTH: usize = 2;

use crate::{
    log::{LogLevel, Logger},
    utils::InterruptGuard,
};

struct BufferData<B> {
    buffer: B,
    line: usize,
    col: usize,
    current_level: LogLevel,
}

impl<B> Write for BufferData<B>
where
    B: ColouredTextBufferWriter + ColouredTextBufferReader + FrameBuffer,
{
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let mut lines = 0;
        let mut current_width = self.col;
        let mut start_char = 0;

        let buffer_width = self.buffer.width();
        let buffer_height = self.buffer.height();

        for (i, char) in text.chars().enumerate() {
            if current_width >= buffer_width || char == '\n' {
                current_width = 0;
                lines += 1;

                if lines >= buffer_height {
                    start_char = i;
                }
            }

            current_width += 1;

            if char == '\t' {
                current_width += TAB_WIDTH;
            }
        }

        let required_shift = (self.line + lines)
            .saturating_sub(buffer_height)
            .min(buffer_height);

        if required_shift > 0 {
            for shift_row in required_shift..self.line {
                let dest_y = shift_row - required_shift;
                let src_y = shift_row;

                if dest_y >= buffer_height {
                    for x in 0..buffer_width {
                        self.buffer.put_coloured(
                            x,
                            dest_y,
                            ' ',
                            TextColour::Black,
                            TextColour::Black,
                        );
                    }
                } else {
                    for x in 0..buffer_width {
                        let (char, fg, bg) = self.buffer.get_coloured(x, src_y);
                        self.buffer.put_coloured(x, dest_y, char, fg, bg);
                    }
                }
            }
        }
        self.line -= required_shift;

        let fg = match self.current_level {
            LogLevel::Debug => TextColour::Gray,
            LogLevel::Warn => TextColour::Yellow,
            LogLevel::Error => TextColour::Red,
            LogLevel::Info => TextColour::Cyan,
        };

        let truncated_text = &text[start_char..];

        for char in truncated_text.chars() {
            if self.col >= buffer_width || char == '\n' {
                for i in self.col..buffer_width {
                    self.buffer.put_coloured(
                        i,
                        self.line,
                        ' ',
                        TextColour::Black,
                        TextColour::Black,
                    );
                }

                self.line += 1;
                self.col = 0;
                continue;
            }

            if char == '\t' {
                for _ in 0..TAB_WIDTH {
                    self.buffer
                        .put_coloured(self.col, self.line, ' ', fg, TextColour::Black);

                    self.col += 1;
                }
            } else {
                self.buffer
                    .put_coloured(self.col, self.line, char, fg, TextColour::Black);

                self.col += 1;
            }
        }

        Ok(())
    }
}

pub struct BufferLogger<B> {
    buffer: InterruptGuard<SpinLock<BufferData<B>>>,
}

impl<B> Logger for BufferLogger<B>
where
    B: ColouredTextBufferWriter + ColouredTextBufferReader + FrameBuffer,
{
    fn log(&self, level: LogLevel, args: core::fmt::Arguments<'_>) {
        let guard = self.buffer.guard();
        let mut data_lock = guard.lock();

        data_lock.current_level = level;
        let _ = data_lock.write_fmt(args);
    }

    fn flush(&self) {}
}

impl<B> BufferLogger<B>
where
    B: ColouredTextBufferWriter + ColouredTextBufferReader + FrameBuffer,
{
    pub fn new(buffer: B) -> Self {
        Self {
            buffer: InterruptGuard::new_lock(BufferData {
                buffer,
                line: 2,
                col: 0,
                current_level: LogLevel::Info,
            }),
        }
    }
}
