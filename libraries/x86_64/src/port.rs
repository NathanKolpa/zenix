use core::arch::asm;
use core::marker::PhantomData;

pub trait WritePort {
    /// # Safety
    ///
    /// Because writing to a port can cause unintended behaviour,
    /// the caller must ensure safety by reading the given datasheet/specification properly.
    unsafe fn write(port: u16, value: Self);
}

pub trait ReadPort {
    /// # Safety
    ///
    /// Because reading from a port can cause unintended behaviour,
    /// the caller must ensure safety by reading the given datasheet/specification properly.
    unsafe fn read(port: u16) -> Self;
}

pub trait WriteAccessMarker {}
pub trait ReadAccessMarker {}

pub struct ReadOnly {}
impl ReadAccessMarker for ReadOnly {}

pub struct WriteOnly {}
impl WriteAccessMarker for WriteOnly {}

pub struct ReadWrite {}
impl ReadAccessMarker for ReadWrite {}
impl WriteAccessMarker for ReadWrite {}

/// A representation of an I/O port for low-level hardware communication.
pub struct Port<Data, Access> {
    port: u16,
    _phantom: PhantomData<(Data, Access)>,
}

impl<Data: ReadPort> Port<Data, ReadOnly> {
    /// # Safety
    ///
    /// The caller must ensure only one instance of each port is constructed
    pub const unsafe fn read_only(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<Data: WritePort> Port<Data, WriteOnly> {
    /// # Safety
    ///
    /// The caller must ensure only one instance of each port is constructed
    pub const unsafe fn write_only(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<Data: ReadPort + WritePort> Port<Data, ReadWrite> {
    /// # Safety
    ///
    /// The caller must ensure only one instance of each port is constructed
    pub const unsafe fn read_write(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<Data, Access> Port<Data, Access> {
    /// # Safety
    ///
    /// See `WritePort`'s safety section.
    #[inline]
    pub unsafe fn write(&mut self, value: Data)
    where
        Access: WriteAccessMarker,
        Data: WritePort,
    {
        Data::write(self.port, value)
    }

    pub unsafe fn write_atomic(&self, value: Data)
    where
        Access: WriteAccessMarker,
        Data: WritePort,
    {
        Data::write(self.port, value)
    }

    /// # Safety
    ///
    /// See `WritePort`'s safety section.
    #[inline]
    pub unsafe fn read(&mut self) -> Data
    where
        Access: ReadAccessMarker,
        Data: ReadPort,
    {
        Data::read(self.port)
    }

    #[inline]
    pub unsafe fn read_atomic(&self) -> Data
    where
        Access: ReadAccessMarker,
        Data: ReadPort,
    {
        Data::read(self.port)
    }
}

impl ReadPort for u8 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        let value: Self;

        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));

        value
    }
}

impl ReadPort for u16 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        let value: Self;

        asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));

        value
    }
}

impl ReadPort for u32 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        let value: Self;

        asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));

        value
    }
}

impl WritePort for u8 {
    #[inline]
    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

impl WritePort for u16 {
    #[inline]
    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

impl WritePort for u32 {
    #[inline]
    unsafe fn write(port: u16, value: Self) {
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }
}
