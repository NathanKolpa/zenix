use crate::{device::Serial, port::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum LineStatusFlags {
    OutputEmpty = 1 << 5,
}

/// More information: [osdev](https://wiki.osdev.org/Serial_Ports)
pub struct Uart16550 {
    data: Port<u8, ReadWrite>,
    interrupts_enabled: Port<u8, WriteOnly>,
    fifo_control: Port<u8, WriteOnly>,
    line_control: Port<u8, WriteOnly>,
    modem_ctrl: Port<u8, WriteOnly>,
    line_status: Port<u8, ReadOnly>,
}

impl Uart16550 {
    ///
    /// # Parameters
    ///
    /// The `base` specifies the base address of the IO port. This value can be one of the
    /// following; each corresponding a a COM port:
    ///
    /// |  COM Port |  IO Port (`base`) | IRQ |
    /// |:---|:---|:--|
    /// | COM1 | 0x3F8 | #4 |
    /// | COM2 | 0x2F8 | #3 |
    /// | COM3 | 0x3E8 | #4 |
    /// | COM4 | 0x2E8 | #3 |
    /// | COM5 | 0x5F8 |  |
    /// | COM6 | 0x4F8 |  |
    /// | COM7 | 0x5E8 |  |
    /// | COM8 | 0x4E8 |  |
    ///
    /// The addresses for COM ports can vary depending on how they are connected to the machine and how the BIOS is configured. Some BIOS configuration utilities allow you to see and set what these are, so if you in doubt for a test machine, this might be a good place to look to get you started.
    /// For the most part, the first two COM ports will be at the addresses specified, the addresses for further COM ports are less reliable.
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

            // Enable interrupts (Bit 1), and interrupt on status change (Bit 3)
            self.interrupts_enabled.write(1 | 1 << 3);
        }
    }

    unsafe fn wait_for(&mut self, status_flag: LineStatusFlags) {
        while (self.line_status.read() & status_flag as u8) == 0 {
            core::hint::spin_loop();
        }
    }
}

impl Serial for Uart16550 {
    fn write_available(&self) -> bool {
        unsafe { self.line_status.read_atomic() & LineStatusFlags::OutputEmpty as u8 != 0 }
    }

    fn write_byte(&mut self, byte: u8) {
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
}
