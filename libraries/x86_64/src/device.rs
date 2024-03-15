mod pic_8259;
mod pit;
pub mod qemu;
mod uart_16550;

mod apic;
mod vga;

pub use apic::*;
pub use pic_8259::*;
pub use pit::*;
pub use uart_16550::*;
pub use vga::*;

pub trait Serial {
    fn write_available(&self) -> bool;
    fn write_byte(&mut self, byte: u8);
}

pub trait FrameBuffer {
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    fn surface_size(&self) -> usize {
        self.width() * self.height()
    }

    fn iter_all_pos(&self) -> impl Iterator<Item = (usize, usize)> + 'static {
        let height = self.height();
        let width = self.width();

        (0..height).flat_map(move |y| (0..width).map(move |x| (x, y)))
    }
}

pub trait TextBuffer {
    fn put(&mut self, x: usize, y: usize, value: char);
    fn put_str(&mut self, x: usize, y: usize, value: &str);
}

#[derive(Clone, Copy)]
pub enum TextBufferColour {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    White,
    Gray,
    Yellow,
}

pub trait ColouredTextBuffer {
    fn put_coloured(
        &mut self,
        x: usize,
        y: usize,
        value: char,
        foreground: TextBufferColour,
        background: TextBufferColour,
    );

    fn put_coloured_str(
        &mut self,
        mut x: usize,
        mut y: usize,
        value: &str,
        foreground: TextBufferColour,
        background: TextBufferColour,
    ) where
        Self: Sized + FrameBuffer,
    {
        let chars = value.chars();
        let width = self.width();

        for char in chars {
            self.put_coloured(x, y, char, foreground, background);

            x += 1;

            if x >= width {
                x = 0;
                y += 1;
            }
        }
    }
}

impl<T> TextBuffer for T
where
    T: ColouredTextBuffer + FrameBuffer,
{
    fn put(&mut self, x: usize, y: usize, value: char) {
        self.put_coloured(
            x,
            y,
            value,
            TextBufferColour::White,
            TextBufferColour::Black,
        );
    }

    fn put_str(&mut self, x: usize, y: usize, value: &str) {
        self.put_coloured_str(
            x,
            y,
            value,
            TextBufferColour::White,
            TextBufferColour::Black,
        );
    }
}
