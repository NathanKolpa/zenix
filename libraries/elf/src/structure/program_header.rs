#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgramHeader<P, F1, F2> {
    // For both 32 and 64 bits the size is 4 bytes. However, the offset in 64 bitmode is 4 bytes,
    // in order to make this work with generics we say that the type is 64 bytes and cut off the
    // last 4 bytes.
    pub kind: u32,
    pub flags_64: F1,
    pub offset: P,
    pub addr: P,
    pub phys_addr: P,
    pub filesize: P,
    pub memsize: P,
    pub flags_32: F2,
    pub align: P,
}
