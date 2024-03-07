use essentials::{nb::BoundedQueue, spin::SpinLock};
use x86_64::{device::Serial, RFlags};

use crate::utils::InterruptGuard;

pub struct SerialChannel<C> {
    queue: BoundedQueue<1024, u8>,
    serial: InterruptGuard<SpinLock<C>>,
}

impl<C> SerialChannel<C>
where
    C: Serial,
{
    pub const fn new(channel: C) -> Self {
        Self {
            serial: InterruptGuard::new_lock(channel),
            queue: BoundedQueue::new(),
        }
    }

    pub fn write_bytes(&self, bytes: &[u8]) {
        let ints_enabled = RFlags::read().interrupts_enabled();

        for byte in bytes {
            if ints_enabled {
                self.write_to_queue(*byte);
            } else {
                self.write_blocking(*byte);
            }
        }
    }

    fn write_to_queue(&self, byte: u8) {
        self.flush_availible();
        self.queue.spin_block_push(byte);
    }

    fn write_blocking(&self, byte: u8) {
        self.flush_availible();

        while self.queue.push(byte).is_err() {
            self.flush_availible();
        }
    }

    pub fn flush_availible(&self) {
        let channel = self.serial.lock();
        let mut channel = channel.lock();

        loop {
            if !channel.write_available() {
                return;
            }

            let Some(byte) = self.queue.pop() else {
                return;
            };

            channel.write_byte(byte);
        }
    }

    pub fn writer(&self) -> impl core::fmt::Write + '_ {
        Writer { channel: self }
    }
}

struct Writer<'a, C> {
    channel: &'a SerialChannel<C>,
}

impl<'a, C> core::fmt::Write for Writer<'a, C>
where
    C: Serial,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.channel.write_bytes(s.as_bytes());
        Ok(())
    }
}
