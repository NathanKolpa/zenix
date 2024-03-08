use core::{
    mem::{size_of, transmute_copy},
    u8,
};

#[repr(C)]
pub struct SDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    oem_creator_id: u32,
    oem_creator_revision: u32,
}

impl SDTHeader {
    fn sum(&self) -> u8 {
        const SIZE: usize = size_of::<SDTHeader>();
        let raw_bytes: [u8; SIZE] = unsafe { transmute_copy(self) };

        raw_bytes
            .into_iter()
            .take(self.length as usize)
            .fold(0u8, |acc, byte| acc.wrapping_add(byte))
    }

    pub fn checksum_ok(&self) -> bool {
        self.sum() == 0
    }
}
