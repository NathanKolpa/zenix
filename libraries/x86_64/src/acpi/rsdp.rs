use core::mem::size_of;

use essentials::address::PhysicalAddress;

use super::sum_bytes;

pub const RSDP_SIGNATURE: [u8; 8] = [b'R', b'S', b'D', b' ', b'P', b'T', b'R', b' '];

#[repr(C)]
pub struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
}

impl RSDP {
    fn sum(&self) -> u8 {
        let raw_bytes = unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, size_of::<RSDP>())
        };

        sum_bytes(raw_bytes.iter().copied())
    }

    pub fn checksum_ok(&self) -> bool {
        self.sum() == 0
    }

    pub fn rsdt_addr(&self) -> PhysicalAddress {
        (self.rsdt_addr as usize).into()
    }

    pub fn oem_id(&self) -> Option<&str> {
        core::str::from_utf8(&self.oem_id).ok()
    }

    pub fn extended(&self) -> bool {
        self.revision >= 2
    }
}
