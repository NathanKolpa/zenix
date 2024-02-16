#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IdentHeader {
    pub magic: [u8; 4],
    pub bits: u8,
    pub endianness: u8,
    pub version: u8,
    _os_abi: u8,
    _abi_version: u8,
    _reserved: [u8; 7],
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

        assert_eq!(ident.bits, 1);
        assert_eq!(ident.endianness, 1);
        assert_eq!(ident.version, 1);
    }

    #[test_case]
    fn test_kernel_elf() {
        let bytes = [
            0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00u8,
        ];

        let ident: IdentHeader = unsafe { transmute_copy(&bytes) };

        assert_eq!(ident.bits, 2);
        assert_eq!(ident.endianness, 1);
        assert_eq!(ident.version, 1);
    }
}
