#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgramHeader<P> {
    // For both 32 and 64 bits the size is 4 bytes. However, the offset in 64 bitmode is 4 bytes,
    // in order to make this work with generics we say that the type is 64 bytes and cut off the
    // last 4 bytes.
    kind: P,
    pub flags: Flags,
    pub offset: P,
    pub addr: P,
    pub phys_addr: P,
    pub filesize: P,
    pub memsize: P,
    pub segment_flags: Flags,
    pub align: P,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flags {
    flags: u32,
}

impl Flags {
    const EXEC_BIT: u32 = 1 << 1;
    const WRITABLE_BIT: u32 = 2 << 1;
    const READABLE_BIT: u32 = 3 << 1;

    pub fn executable(&self) -> bool {
        (self.flags & Self::EXEC_BIT) != 0
    }

    pub fn readable(&self) -> bool {
        (self.flags & Self::READABLE_BIT) != 0
    }

    pub fn writable(&self) -> bool {
        (self.flags & Self::WRITABLE_BIT) != 0
    }
}

impl ProgramHeader<u32> {
    pub fn kind(&self) -> SegmentKind {
        self.kind.into()
    }
}

impl ProgramHeader<u64> {
    pub fn kind(&self) -> SegmentKind {
        (self.kind as u32).into()
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentKind {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Intererp = 3,
    Note = 4,
    ProgramHeader = 6,
    ThreadLocalStorage = 7,
    Other,
}

impl From<u32> for SegmentKind {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Null,
            1 => Self::Load,
            2 => Self::Dynamic,
            3 => Self::Intererp,
            4 => Self::Note,
            6 => Self::ProgramHeader,
            7 => Self::ThreadLocalStorage,
            _ => Self::Other,
        }
    }
}
