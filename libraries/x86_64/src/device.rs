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

pub trait TextBufferWriter {
    fn put(&mut self, x: usize, y: usize, value: char);
    fn put_str(&mut self, x: usize, y: usize, value: &str);
}

pub trait TextBufferReader {
    fn get(&self, x: usize, y: usize) -> char;
}

#[derive(Clone, Copy)]
pub enum TextColour {
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

pub trait ColouredTextBufferWriter {
    fn put_coloured(
        &mut self,
        x: usize,
        y: usize,
        value: char,
        foreground: TextColour,
        background: TextColour,
    );

    fn put_coloured_str(
        &mut self,
        mut x: usize,
        mut y: usize,
        value: &str,
        foreground: TextColour,
        background: TextColour,
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

pub trait ColouredTextBufferReader {
    fn get_coloured(&self, x: usize, y: usize) -> (char, TextColour, TextColour);
}

impl<T> TextBufferWriter for T
where
    T: ColouredTextBufferWriter + FrameBuffer,
{
    fn put(&mut self, x: usize, y: usize, value: char) {
        self.put_coloured(x, y, value, TextColour::White, TextColour::Black);
    }

    fn put_str(&mut self, x: usize, y: usize, value: &str) {
        self.put_coloured_str(x, y, value, TextColour::White, TextColour::Black);
    }
}

impl<T> TextBufferReader for T
where
    T: ColouredTextBufferReader,
{
    fn get(&self, x: usize, y: usize) -> char {
        let (char, _, _) = self.get_coloured(x, y);
        char
    }
}
