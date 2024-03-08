use core::mem::{size_of, transmute_copy};

pub const SIGNATURE: [u8; 8] = [b'R', b'S', b'D', b' ', b'P', b'T', b'R', b' '];

#[repr(C)]
pub struct RSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_addr: u32,
}

impl RSDP {
    fn sum(&self) -> u8 {
        const SIZE: usize = size_of::<RSDP>();
        let raw_bytes: [u8; SIZE] = unsafe { transmute_copy(self) };

        raw_bytes
            .into_iter()
            .fold(0u8, |acc, byte| acc.wrapping_add(byte))
    }

    pub fn checksum_ok(&self) -> bool {
        self.sum() == 0
    }
}

#[repr(C)]
pub struct ExtendedRSDP {
    pub rsdp: RSDP,
    pub length: u32,
    pub extended_rsdt_addr: u64,
    pub extended_checksum: u8,
    pub _reserved: [u8; 3],
}
