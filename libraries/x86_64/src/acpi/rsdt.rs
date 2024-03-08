pub const SIGNATURE: [u8; 8] = [b'R', b'S', b'D', b' ', b'P', b'T', b'R', b' '];

#[repr(C)]
pub struct RSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_addr: u32,
}

#[repr(C)]
pub struct ExtendedRSDP {
    pub rsdp: RSDP,
    pub length: u32,
    pub extended_rsdt_addr: u64,
    pub extended_checksum: u8,
    pub _reserved: [u8; 3],
}
