use x86_64::port::Port;

pub fn set_fail_msg(message: &str) {
    let mut cursor = msg("Pre kernel panic: ", 0, VGA_WHITE, VGA_RED);
    cursor = msg(message, cursor, VGA_WHITE, VGA_RED);
    msg(
        "Halting boot procedure...",
        cursor + (VGA_WIDTH - cursor % VGA_WIDTH),
        VGA_WHITE,
        VGA_RED,
    );
}

pub fn set_running_msg() {
    msg("Running pre kernel...", 0, VGA_WHITE, VGA_BLACK);
}

pub fn clear_screen() {
    for i in 0..VGA_SIZE {
        write_cell(i, b' ', VGA_BLACK, VGA_BLACK);
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

fn msg(message: &str, start: usize, fg: u8, bg: u8) -> usize {
    let mut cursor = start;
    for char in message.bytes().take(VGA_SIZE) {
        write_cell(cursor, char, fg, bg);
        cursor = cursor.wrapping_add(1);
    }
    cursor
}

const VGA_HEIGHT: usize = 25;
const VGA_WIDTH: usize = 80;
const VGA_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;

const VGA_WHITE: u8 = 15;
const VGA_BLACK: u8 = 0;
const VGA_RED: u8 = 4;

fn write_cell(pos: usize, char: u8, fg: u8, bg: u8) {
    const SCREEN_PTR: *mut u16 = 0xB8000 as *mut u16;

    if pos > VGA_SIZE {
        return;
    }

    const COLOR_MASK: u8 = 0x0F;
    let color = ((bg & COLOR_MASK) << 4) | (fg & COLOR_MASK);

    let ascii_char = if char.is_ascii() { char } else { b'?' };
    let cell = (ascii_char as u16) | ((color as u16) << 8);

    let cell_ptr = unsafe { SCREEN_PTR.add(pos) };

    unsafe { core::ptr::write_volatile(cell_ptr, cell) };
}
