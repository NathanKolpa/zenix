use core::mem::size_of;

use crate::{
    reader::{header::*, ReadError},
    structure::ident::*,
};

pub struct ElfReader<'a> {
    raw_data: &'a [u8],
    ident_header: IdentHeader,
}

impl<'a> ElfReader<'a> {
    fn magic_matches(raw_data: &[u8]) -> bool {
        raw_data[0..4] == [0x7f, 0x45, 0x4c, 0x46]
    }

    pub fn new(raw_data: &'a [u8]) -> Result<Self, ReadError> {
        if raw_data.len() < size_of::<IdentHeader>() {
            return Err(ReadError::TooSmall);
        }

        if !Self::magic_matches(raw_data) {
            return Err(ReadError::InvalidMagic);
        }

        let ident_header = unsafe { *(raw_data.as_ptr() as *const IdentHeader) };

        Ok(Self {
            raw_data,
            ident_header,
        })
    }

    pub fn version(&self) -> IdentVersion {
        self.ident_header.version
    }

    pub fn header(&self) -> Result<ArchHeaderReader<'a>, ReadError> {
        Ok(match self.ident_header.bits {
            Bits::Bits32 => ArchHeaderReader::Bits32(ElfHeaderReader::new(self.raw_data)?),
            Bits::Bits64 => ArchHeaderReader::Bits64(ElfHeaderReader::new(self.raw_data)?),
        })
    }
}
