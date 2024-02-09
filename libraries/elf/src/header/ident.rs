#[repr(C)]
pub struct IdentHeader {
    bits: Bits,
    endianness: Endianness,
    version: Version,
    os_abi: u8,
    abi_version: u8,
    _padding: u8,
    kind: ObjectKind,
    arch: Arch,
}

#[repr(u8)]
pub enum Version {
    Original = 1,
    Other,
}

#[repr(u8)]
pub enum Bits {
    Bits32 = 1,
    Bits64 = 2,
}

#[repr(u8)]
pub enum Endianness {
    Little,
    Big,
}

#[repr(u8)]
pub enum ObjectKind {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    SharedObject = 3,
    Other,
}

#[repr(u8)]
pub enum Arch {
    Archx86 = 0x03,
    Archx86_64 = 0x3E,
    Other,
}
