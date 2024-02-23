use core::mem::size_of;

use essentials::address::VirtualAddress;
use x86_64::port::Port;

pub fn set_fail_msg(message: &str) {
    let mut cursor = msg("Failure", STATUS_START, VGA_RED, VGA_BLACK);
    cursor = msg("Reason: ", next_line(cursor), VGA_RED, VGA_BLACK);
    cursor = msg(message, cursor, VGA_RED, VGA_BLACK);
    msg(
        "Halting CPU. Reset your machine to try again.",
        next_line(cursor),
        VGA_RED,
        VGA_BLACK,
    );
}

const RUNNING_MSG: &str = "Running pre kernel: ";
const STATUS_START: usize = RUNNING_MSG.len();

pub fn set_running_msg() {
    msg(RUNNING_MSG, 0, VGA_WHITE, VGA_BLACK);
}

pub fn set_success_msg() {
    let cursor = msg("Ok", STATUS_START, VGA_GREEN, VGA_BLACK);

    msg(
        "Entering the Zenix kernel...",
        next_line(cursor),
        VGA_WHITE,
        VGA_BLACK,
    );
}

const fn next_line(cursor: usize) -> usize {
    (cursor / VGA_WIDTH + 1) * VGA_WIDTH
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

pub const VGA_ADDR: VirtualAddress = VirtualAddress::new(0xB8000);
pub const VGA_LEN: usize = VGA_SIZE * size_of::<u16>();

const VGA_WHITE: u8 = 15;
const VGA_BLACK: u8 = 0;
const VGA_RED: u8 = 12;
const VGA_GREEN: u8 = 2;

fn write_cell(pos: usize, char: u8, fg: u8, bg: u8) {
    const SCREEN_PTR: *mut u16 = VGA_ADDR.as_mut_ptr();

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
