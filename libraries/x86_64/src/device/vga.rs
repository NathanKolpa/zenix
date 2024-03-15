use core::mem::size_of;

use essentials::address::VirtualAddress;

use crate::device::{ColouredTextBufferReader, ColouredTextBufferWriter, FrameBuffer, TextColour};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

#[repr(C)]
#[derive(Clone, Copy)]
struct VgaColour {
    code: u8,
}

impl VgaColour {
    const fn map_colour(colour: TextColour) -> u8 {
        match colour {
            TextColour::Black => 0,
            TextColour::Blue => 1,
            TextColour::Green => 2,
            TextColour::Cyan => 3,
            TextColour::Red => 4,
            TextColour::Magenta => 5,
            TextColour::Brown => 6,
            TextColour::Gray => 8,
            TextColour::Yellow => 14,
            TextColour::White => 15,
        }
    }

    const fn from_colours(foreground: TextColour, background: TextColour) -> Self {
        Self {
            code: (Self::map_colour(background) << 4) | Self::map_colour(foreground),
        }
    }

    const fn map_text_colour(code: u8) -> TextColour {
        match code {
            1 => TextColour::Blue,
            2 => TextColour::Green,
            3 => TextColour::Cyan,
            4 => TextColour::Red,
            5 => TextColour::Magenta,
            6 => TextColour::Brown,
            8 => TextColour::Gray,
            14 => TextColour::Yellow,
            15 => TextColour::White,
            _ => TextColour::Black,
        }
    }

    fn background(&self) -> TextColour {
        Self::map_text_colour(self.code >> 4)
    }
    fn foreground(&self) -> TextColour {
        Self::map_text_colour(self.code & 0xF)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VgaCell {
    character: u8,
    colour: VgaColour,
}

pub struct VgaBuffer {
    buffer: &'static mut [[VgaCell; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl VgaBuffer {
    pub const WIDTH: usize = BUFFER_WIDTH;
    pub const HEIGHT: usize = BUFFER_HEIGHT;

    pub const BUFFER_ADDR: VirtualAddress = VirtualAddress::new(0xb8000);
    pub const BUFFER_SIZE: usize = BUFFER_WIDTH * BUFFER_HEIGHT * size_of::<VgaCell>();

    pub const unsafe fn new() -> VgaBuffer {
        VgaBuffer {
            buffer: &mut *Self::BUFFER_ADDR.as_mut_ptr(),
        }
    }
}

impl FrameBuffer for VgaBuffer {
    fn width(&self) -> usize {
        BUFFER_WIDTH
    }

    fn height(&self) -> usize {
        BUFFER_HEIGHT
    }
}

impl ColouredTextBufferWriter for VgaBuffer {
    fn put_coloured(
        &mut self,
        x: usize,
        y: usize,
        value: char,
        foreground: super::TextColour,
        background: super::TextColour,
    ) {
        let character = if value.is_ascii() { value as u8 } else { b'?' };

        let cell = VgaCell {
            character,
            colour: VgaColour::from_colours(foreground, background),
        };

        if let Some(cell_ref) = self.buffer.get_mut(y).and_then(|rows| rows.get_mut(x)) {
            *cell_ref = cell;
        }
    }
}

impl ColouredTextBufferReader for VgaBuffer {
    fn get_coloured(&self, x: usize, y: usize) -> (char, TextColour, TextColour) {
        let cell = self.buffer[y][x];

        (
            char::from(cell.character),
            cell.colour.foreground(),
            cell.colour.background(),
        )
    }
}
