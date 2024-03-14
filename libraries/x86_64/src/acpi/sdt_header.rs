use super::sum_bytes;

#[repr(packed, C)]
#[derive(Debug, Clone, Copy)]
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
        let raw_bytes = unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, self.length as usize)
        };

        sum_bytes(raw_bytes.iter().copied())
    }

    pub fn checksum_ok(&self) -> bool {
        self.sum() == 0
    }

    pub fn length(&self) -> usize {
        self.length as usize
    }

    pub fn signature(&self) -> Option<&str> {
        core::str::from_utf8(&self.signature).ok()
    }
}
