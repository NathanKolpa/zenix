use crate::port::*;

#[repr(u8)]
enum Command {
    Init = 0x11,
    EndOfInt = 0x20,
    Mode8086 = 0x01,
}

pub struct Pic8259 {
    interrupt_offset: u8,
    command: Port<u8, WriteOnly>,
    data: Port<u8, ReadWrite>,
}

impl Pic8259 {
    unsafe fn write_offset(&mut self) {
        self.write_data(self.interrupt_offset);
    }

    unsafe fn write_command(&mut self, cmd: Command) {
        self.command.write(cmd as u8);
    }

    unsafe fn write_data(&mut self, data: u8) {
        self.data.write(data);
    }

    unsafe fn read_mask(&mut self) -> u8 {
        self.data.read()
    }

    unsafe fn write_mask(&mut self, mask: u8) {
        self.data.write(mask)
    }

    fn handles_interrupt(&self, interrupt: u8) -> bool {
        self.interrupt_offset <= interrupt && interrupt < self.interrupt_offset + 8
    }
}

pub struct ChainedPic8259 {
    pics: [Pic8259; 2],
}

impl ChainedPic8259 {
    pub const unsafe fn new(interrupt_offset: u8) -> Self {
        Self {
            pics: [
                Pic8259 {
                    interrupt_offset,
                    command: Port::write_only(0x20),
                    data: Port::read_write(0x21),
                },
                Pic8259 {
                    interrupt_offset: interrupt_offset + 8,
                    command: Port::write_only(0xA0),
                    data: Port::read_write(0xA1),
                },
            ],
        }
    }

    pub fn init(&mut self) {
        let mut wait_port = unsafe { Port::write_only(0x80) };

        unsafe {
            let mut wait = || wait_port.write(0_u8);

            let read_mask_1 = self.pics[0].read_mask();
            let read_mask_2 = self.pics[1].read_mask();

            self.pics[0].write_command(Command::Init);
            wait();
            self.pics[1].write_command(Command::Init);
            wait();

            self.pics[0].write_offset();
            wait();
            self.pics[1].write_offset();
            wait();

            self.pics[0].write_data(4);
            wait();
            self.pics[1].write_data(2);
            wait();

            self.pics[0].write_command(Command::Mode8086);
            wait();
            self.pics[1].write_command(Command::Mode8086);
            wait();

            self.pics[0].write_mask(read_mask_1);
            self.pics[1].write_mask(read_mask_2);
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            self.pics[0].write_mask(0);
            self.pics[1].write_mask(0);
        }
    }

    pub fn disable(&mut self) {
        unsafe {
            self.pics[0].write_mask(0xff);
            self.pics[1].write_mask(0xff);
        }
    }

    pub fn end_of_interrupt(&mut self, interrupt: u8) {
        let pic = self
            .pics
            .iter_mut()
            .find(|x| x.handles_interrupt(interrupt));

        if let Some(pic) = pic {
            unsafe {
                pic.write_command(Command::EndOfInt);
            }
        }
    }
}
