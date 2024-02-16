use crate::{
    reader::{header::*, ElfReadError},
    structure::ident::*,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdentVersion {
    Original,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bits {
    Bits32,
    Bits64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Clone)]
pub struct ElfReader<'a> {
    raw_data: &'a [u8],
    ident_header: IdentHeader,
    endianness: Endianness,
    bits: Bits,
}

impl<'a> ElfReader<'a> {
    fn magic_matches(raw_data: &[u8]) -> bool {
        raw_data[0..4] == [0x7f, b'E', b'L', b'F']
    }

    pub fn new(raw_data: &'a [u8]) -> Result<Self, ElfReadError> {
        if !Self::magic_matches(raw_data) {
            return Err(ElfReadError::InvalidMagic);
        }

        let ident_header: IdentHeader = unsafe { super::read_struct(raw_data, 0) }?;

        let endianness = match ident_header.endianness {
            1 => Endianness::Little,
            2 => Endianness::Big,
            _ => return Err(ElfReadError::InvalidEndianness),
        };

        let bits = match ident_header.bits {
            1 => Bits::Bits32,
            2 => Bits::Bits64,
            _ => return Err(ElfReadError::UnsupportedBits),
        };

        Ok(Self {
            raw_data,
            ident_header,
            bits,
            endianness,
        })
    }

    pub fn version(&self) -> IdentVersion {
        match self.ident_header.version {
            1 => IdentVersion::Original,
            _ => IdentVersion::Other,
        }
    }

    pub fn bits(&self) -> Bits {
        self.bits
    }

    pub fn endianness(&self) -> Endianness {
        self.endianness
    }

    pub fn header(&self) -> Result<ArchHeaderReader<'a>, ElfReadError> {
        let raw_data = self.raw_data;

        match self.bits() {
            Bits::Bits32 => Ok(ArchHeaderReader::Bits32(ElfHeaderReader::new(raw_data)?)),
            Bits::Bits64 => Ok(ArchHeaderReader::Bits64(ElfHeaderReader::new(raw_data)?)),
        }
    }
}
