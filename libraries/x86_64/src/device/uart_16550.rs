use crate::port::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum LineStatusFlags {
    OutputEmpty = 1 << 5,
}

pub struct Uart16550 {
    data: Port<u8, ReadWrite>,
    interrupts_enabled: Port<u8, WriteOnly>,
    fifo_control: Port<u8, WriteOnly>,
    line_control: Port<u8, WriteOnly>,
    modem_ctrl: Port<u8, WriteOnly>,
    line_status: Port<u8, ReadOnly>,
}

impl core::fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.write(byte)
        }
        Ok(())
    }
}

impl Uart16550 {
    pub const unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::read_write(base),
            interrupts_enabled: Port::write_only(base + 1),
            fifo_control: Port::write_only(base + 2),
            line_control: Port::write_only(base + 3),
            modem_ctrl: Port::write_only(base + 4),
            line_status: Port::read_only(base + 5),
        }
    }

    pub unsafe fn new_and_init(base: u16) -> Self {
        let mut uart = Self::new(base);
        uart.init();
        uart
    }

    fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.interrupts_enabled.write(0x00);

            // Enable DLAB
            self.line_control.write(0x80);

            // Set maximum speed to 38400 bps by configuring DLL and DLM
            self.data.write(0x03);
            self.interrupts_enabled.write(0x00);

            // Disable DLAB and set data word length to 8 bits
            self.line_control.write(0x03);

            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            self.fifo_control.write(0xC7);

            // Mark data terminal ready, signal request to send
            // and enable auxiliary output #2 (used as interrupt line for CPU)
            self.modem_ctrl.write(0x0B);

            // Enable  interrupts
            // self.interrupts_enabled.write(0x01);
        }
    }

    fn write(&mut self, byte: u8) {
        unsafe {
            match byte {
                8 | 0x7F => {
                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(8);

                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(b' ');

                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(8);
                }
                _ => {
                    self.wait_for(LineStatusFlags::OutputEmpty);
                    self.data.write(byte);
                }
            }
        }
    }

    unsafe fn wait_for(&mut self, status_flag: LineStatusFlags) {
        while (self.line_status.read() & status_flag as u8) == 0 {
            core::hint::spin_loop();
        }
    }
}
