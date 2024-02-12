#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdentHeader {
    pub magic: [u8; 4],
    pub bits: Bits,
    pub endianness: Endianness,
    pub version: IdentVersion,
    _os_abi: u8,
    _abi_version: u8,
    _reserved: [u8; 7],
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdentVersion {
    Original = 1,
    Other,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bits {
    Bits32 = 1,
    Bits64 = 2,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    Little = 1,
    Big = 2,
}

#[cfg(test)]
mod tests {
    use core::mem::transmute_copy;

    use super::*;

    #[test_case]
    fn test_pre_kernel_elf() {
        let bytes = [
            0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00u8,
        ];

        let ident: IdentHeader = unsafe { transmute_copy(&bytes) };

        assert_eq!(ident.bits, Bits::Bits32);
        assert_eq!(ident.endianness, Endianness::Little);
        assert_eq!(ident.version, IdentVersion::Original);
    }

    #[test_case]
    fn test_kernel_elf() {
        let bytes = [
            0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00u8,
        ];

        let ident: IdentHeader = unsafe { transmute_copy(&bytes) };

        assert_eq!(ident.bits, Bits::Bits64);
        assert_eq!(ident.endianness, Endianness::Little);
        assert_eq!(ident.version, IdentVersion::Original);
    }
}
