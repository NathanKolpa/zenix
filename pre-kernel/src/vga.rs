use essentials::address::VirtualAddress;
use x86_64::{
    device::{ColouredTextBufferWriter, FrameBuffer, TextColour, VgaBuffer},
    port::Port,
};

pub fn set_fail_msg(message: &str) {
    let mut cursor = msg("Failure", STATUS_START, TextColour::Red, TextColour::Black);

    cursor = msg(
        "Reason: ",
        next_line(cursor),
        TextColour::Red,
        TextColour::Black,
    );

    cursor = msg(message, cursor, TextColour::Red, TextColour::Black);

    msg(
        "Halting CPU. Reset your machine to try again.",
        next_line(cursor),
        TextColour::Red,
        TextColour::Black,
    );
}

const RUNNING_MSG: &str = "Running Pre-kernel: ";
const STATUS_START: usize = RUNNING_MSG.len();

pub fn set_running_msg() {
    msg(RUNNING_MSG, 0, TextColour::White, TextColour::Black);
}

pub fn set_success_msg() {
    let cursor = msg("Ok", STATUS_START, TextColour::Green, TextColour::Black);

    msg(
        "Entering the Kernel",
        next_line(cursor),
        TextColour::White,
        TextColour::Black,
    );
}

const fn next_line(cursor: usize) -> usize {
    (cursor / VgaBuffer::HEIGHT + 1) * VgaBuffer::WIDTH
}

pub fn clear_screen() {
    let mut buffer = unsafe { VgaBuffer::new() };

    for (x, y) in buffer.iter_all_pos() {
        buffer.put_coloured(x, y, ' ', TextColour::Black, TextColour::Black);
    }

    disable_cursor();
}

fn disable_cursor() {
    unsafe {
        let mut port1 = Port::write_only(0x3D4);
        let mut port2 = Port::write_only(0x3D5);

        port1.write(0x0Au8);
        port2.write(0x20u8);
    }
}

fn msg(message: &str, start: usize, fg: TextColour, bg: TextColour) -> usize {
    let mut buffer = unsafe { VgaBuffer::new() };

    buffer.put_coloured_str(
        start % buffer.width(),
        start / buffer.width(),
        message,
        fg,
        bg,
    );

    start + message.len()
}

pub const VGA_ADDR: VirtualAddress = VgaBuffer::BUFFER_ADDR;
pub const VGA_LEN: usize = VgaBuffer::BUFFER_SIZE;
