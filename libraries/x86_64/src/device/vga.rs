use core::mem::size_of;

use essentials::address::VirtualAddress;

use crate::device::{ColouredTextBuffer, FrameBuffer, TextBufferColour};

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

#[repr(C)]
struct VgaColour {
    code: u8,
}

impl VgaColour {
    const fn map_colour(colour: TextBufferColour) -> u8 {
        match colour {
            TextBufferColour::Black => 0,
            TextBufferColour::Blue => 1,
            TextBufferColour::Green => 2,
            TextBufferColour::Cyan => 3,
            TextBufferColour::Red => 4,
            TextBufferColour::Magenta => 5,
            TextBufferColour::Brown => 6,
            TextBufferColour::Gray => 8,
            TextBufferColour::Yellow => 14,
            TextBufferColour::White => 15,
        }
    }

    const fn from_colours(foreground: TextBufferColour, background: TextBufferColour) -> Self {
        Self {
            code: (Self::map_colour(background) << 4) | Self::map_colour(foreground),
        }
    }
}

#[repr(C)]
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

impl ColouredTextBuffer for VgaBuffer {
    fn put_coloured(
        &mut self,
        x: usize,
        y: usize,
        value: char,
        foreground: super::TextBufferColour,
        background: super::TextBufferColour,
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
