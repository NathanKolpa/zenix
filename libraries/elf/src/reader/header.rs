use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
    usize,
};

use crate::{
    reader::{program_header::ProgramHeaderReader, ReadError},
    structure::{header::*, ident::IdentHeader, program_header::ProgramHeader},
};

pub use crate::structure::header::{Arch, ElfVersion, ObjectKind};

pub enum ArchHeaderReader<'a> {
    Bits32(ElfHeaderReader<'a, u32>),
    Bits64(ElfHeaderReader<'a, u64>),
}

pub struct ElfHeaderReader<'a, P> {
    raw_data: &'a [u8],
    header: ElfHeader<P>,
}

impl<'a, P: Copy + TryInto<usize>> ElfHeaderReader<'a, P> {
    pub fn new(raw_data: &'a [u8]) -> Result<Self, ReadError> {
        let offset = size_of::<IdentHeader>();
        let header_slice = &raw_data[offset..];

        if header_slice.len() < size_of::<ElfHeader<P>>() {
            return Err(ReadError::TooSmall);
        }

        let header = unsafe { *(header_slice.as_ptr() as *const ElfHeader<P>) };

        Ok(Self { raw_data, header })
    }

    pub fn version(&self) -> ElfVersion {
        self.header.version
    }

    pub fn arch(&self) -> Arch {
        self.header.arch
    }

    pub fn object_kind(&self) -> ObjectKind {
        self.header.kind
    }

    pub fn entry_point(&self) -> Option<P> {
        match self.object_kind() {
            ObjectKind::Executable => Some(self.header.entrypoint),
            _ => None,
        }
    }

    pub fn program_headers(
        &self,
    ) -> Result<impl Iterator<Item = ProgramHeaderReader<'a, P>>, ReadError> {
        if self.header.program_header_table_entry_size as usize != size_of::<ProgramHeader<P>>() {
            return Err(ReadError::InvalidEntrySize);
        }

        let offset: usize = self
            .header
            .program_header_ptr
            .try_into()
            .map_err(|_| ReadError::TooSmall)?;

        let min_size = offset
            + self.header.program_header_table_entry_size as usize
                * self.header.program_header_table_len as usize;

        let align = align_of::<ProgramHeader<P>>();
        if offset % align != 0 {
            return Err(ReadError::NotAligned);
        }

        if self.raw_data.len() < min_size {
            return Err(ReadError::TooSmall);
        }

        Ok(ProgramHeaderIter {
            raw_data: self.raw_data,
            offset,
            count: self.header.program_header_table_len,
            current: 0,
            size: self.header.program_header_table_entry_size,
            _phantom: PhantomData,
        })
    }
}

struct ProgramHeaderIter<'a, P> {
    raw_data: &'a [u8],
    offset: usize,
    count: u16,
    current: u16,
    size: u16,
    _phantom: PhantomData<P>,
}

impl<'a, P: Copy> Iterator for ProgramHeaderIter<'a, P> {
    type Item = ProgramHeaderReader<'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }

        let entry_offset = self.offset + self.current as usize * self.size as usize;
        self.current += 1;

        Some(unsafe { ProgramHeaderReader::new_unchecked(self.raw_data, entry_offset) })
    }
}
