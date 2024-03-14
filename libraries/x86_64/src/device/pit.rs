use core::time::Duration;

use crate::port::{Port, ReadWrite, WriteOnly};

pub struct Pit {
    channel0: Port<u8, ReadWrite>,
    _channel1: Port<u8, ReadWrite>,
    _channel2: Port<u8, ReadWrite>,
    command: Port<u8, WriteOnly>,
}

impl Pit {
    pub const unsafe fn new() -> Self {
        Self {
            channel0: Port::read_write(0x40),
            _channel1: Port::read_write(0x41),
            _channel2: Port::read_write(0x42),
            command: Port::write_only(0x43),
        }
    }

    pub fn set_freq(&mut self, freq: u32) {
        let divisor = (1193182 / freq) as u16;
        let [lower, higher] = divisor.to_ne_bytes();

        unsafe {
            self.command.write(0x36);

            self.channel0.write(lower);
            self.channel0.write(higher);
        }
    }

    pub fn set_interval(&mut self, interval: Duration) {
        self.set_freq(1000_000 / interval.as_micros() as u32);
    }
}
