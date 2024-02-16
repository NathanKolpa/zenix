use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
};

use essentials::address::VirtualAddress;

use crate::{
    reader::{
        program_header::ProgramHeaderReader, section_header::SectionHeaderReader, ElfReadError,
    },
    structure::{
        header::*, ident::IdentHeader, program_header::ProgramHeader, segment_header::SectionHeader,
    },
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RelocationTableEntry<P> {
    offset: P,
    info: P,
    addend: P,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelocationEntryKind {
    Relative,
    Other,
}

impl<P: Copy + TryInto<usize>> RelocationTableEntry<P> {
    pub fn kind(&self) -> RelocationEntryKind {
        let kind: Result<usize, _> = self.info.try_into();

        match kind {
            Ok(8) => RelocationEntryKind::Relative,
            _ => RelocationEntryKind::Other,
        }
    }

    pub fn offset(&self) -> P {
        self.offset
    }

    pub fn addend(&self) -> P {
        self.addend
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectKind {
    None,
    Relocatable,
    Executable,
    SharedObject,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arch {
    Archx86,
    Archx86_64,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElfVersion {
    Original,
    Other,
}

pub enum ArchHeaderReader<'a> {
    Bits32(ElfHeaderReader<'a, u32>),
    Bits64(ElfHeaderReader<'a, u64>),
}

pub struct ElfHeaderReader<'a, P> {
    raw_data: &'a [u8],
    header: ElfHeader<P>,
}

impl<'a, P: Copy + TryInto<usize> + TryInto<u32>> ElfHeaderReader<'a, P> {
    pub fn new(raw_data: &'a [u8]) -> Result<Self, ElfReadError> {
        let offset = size_of::<IdentHeader>();
        let header = unsafe { super::read_struct(raw_data, offset) }?;

        Ok(Self { raw_data, header })
    }

    pub fn elf_start(&self) -> VirtualAddress {
        VirtualAddress::from(self.raw_data.as_ptr())
    }

    pub fn elf_end(&self) -> VirtualAddress {
        self.elf_start() + self.raw_data.len()
    }

    pub fn version(&self) -> ElfVersion {
        match self.header.version {
            1 => ElfVersion::Original,
            _ => ElfVersion::Other,
        }
    }

    pub fn arch(&self) -> Arch {
        match self.header.arch {
            0x03 => Arch::Archx86,
            0x3E => Arch::Archx86_64,
            _ => Arch::Other,
        }
    }

    pub fn object_kind(&self) -> ObjectKind {
        match self.header.kind {
            0 => ObjectKind::None,
            1 => ObjectKind::Relocatable,
            2 => ObjectKind::Executable,
            3 => ObjectKind::SharedObject,
            _ => ObjectKind::Other,
        }
    }

    pub fn entry_point(&self) -> Option<P> {
        let numeric_value: usize = self.header.entrypoint.try_into().ok()?;
        if numeric_value == 0 {
            return None;
        }

        Some(self.header.entrypoint)
    }

    pub fn section_names(&self) -> Result<SectionHeaderReader<'a, P>, ElfReadError> {
        let offset: usize = self
            .header
            .section_header_offset
            .try_into()
            .map_err(|_| ElfReadError::TooSmall)?;

        let entry_size = self.header.section_header_table_entry_size as usize;
        let len = self.header.section_header_table_len as usize;

        self.section_array_valid::<SectionHeader<P>>(offset, entry_size, len)?;

        SectionHeaderReader::new(self.raw_data, offset + entry_size * len)
    }

    fn section_array_valid<T>(
        &self,
        offset: usize,
        entry_size: usize,
        len: usize,
    ) -> Result<(), ElfReadError> {
        if entry_size != size_of::<T>() {
            return Err(ElfReadError::InvalidEntrySize);
        }

        let min_size = offset + entry_size * len;

        let align = align_of::<T>();
        if offset % align != 0 {
            return Err(ElfReadError::NotAligned);
        }

        if self.raw_data.len() < min_size {
            return Err(ElfReadError::TooSmall);
        }

        Ok(())
    }

    fn relocation_table_inner<F1: Copy, F2: Copy>(
        &self,
    ) -> Result<Option<&'a [RelocationTableEntry<P>]>, ElfReadError> {
        let relocation_table = self.program_headers_inner::<F1, F2>()?.find_map(|header| {
            match header.data().ok()? {
                crate::SegmentData::Dynamic(dyn_data) => dyn_data.relocation_table_entries(),
                _ => None,
            }
        });

        let Some((table_ptr, table_size, entry_size)) = relocation_table else {
            return Ok(None);
        };

        let entry_size: usize = entry_size.try_into().map_err(|_| ElfReadError::TooSmall)?;
        let table_size: usize = table_size.try_into().map_err(|_| ElfReadError::TooSmall)?;
        let table_ptr: usize = table_ptr.try_into().map_err(|_| ElfReadError::TooSmall)?;

        if entry_size != size_of::<RelocationTableEntry<P>>() {
            return Err(ElfReadError::InvalidEntrySize);
        }

        if table_size % size_of::<RelocationTableEntry<P>>() != 0 {
            return Err(ElfReadError::NotAligned);
        }

        let table_len = table_size / entry_size;

        let table_section = self.program_headers_inner::<F1, F2>()?.find(|header| {
            let addr: Result<usize, _> = header.addr().try_into();
            let size: Result<usize, _> = header.file_size().try_into();

            match (addr, size) {
                (Ok(addr), Ok(size)) => table_ptr >= addr && table_ptr <= addr + size,
                _ => false,
            }
        });

        let Some(table_section) = table_section else {
            return Err(ElfReadError::TooSmall);
        };

        let offset_addr: usize = table_section
            .addr()
            .try_into()
            .map_err(|_| ElfReadError::TooSmall)?;

        let bytes = table_section.bytes()?;

        let table_index = table_ptr - offset_addr;
        let table_bytes = &bytes[table_index..(table_index + table_size)];

        Ok(Some(unsafe {
            core::slice::from_raw_parts(table_bytes.as_ptr() as *const _, table_len)
        }))
    }

    fn program_headers_inner<F1: Copy, F2: Copy>(
        &self,
    ) -> Result<impl Iterator<Item = ProgramHeaderReader<'a, P, F1, F2>>, ElfReadError> {
        let offset: usize = self
            .header
            .program_header_ptr
            .try_into()
            .map_err(|_| ElfReadError::TooSmall)?;

        self.section_array_valid::<ProgramHeader<P, F1, F2>>(
            offset,
            self.header.program_header_table_entry_size as usize,
            self.header.program_header_table_len as usize,
        )?;

        Ok(ProgramHeaderIter {
            raw_data: self.raw_data,
            offset,
            count: self.header.program_header_table_len,
            current: 0,
            size: self.header.program_header_table_entry_size,
            _phantom: PhantomData::<(P, F1, F2)>,
        })
    }

    pub fn section_headers(
        &self,
    ) -> Result<impl Iterator<Item = SectionHeaderReader<'a, P>>, ElfReadError> {
        let offset: usize = self
            .header
            .section_header_offset
            .try_into()
            .map_err(|_| ElfReadError::TooSmall)?;

        self.section_array_valid::<SectionHeaderReader<P>>(
            offset,
            self.header.section_header_table_entry_size as usize,
            self.header.section_header_table_len as usize,
        )?;

        Ok(SectionHeaderIter {
            raw_data: self.raw_data,
            offset,
            count: self.header.program_header_table_len,
            current: 0,
            size: self.header.program_header_table_entry_size,
            _phantom: PhantomData,
        })
    }
}

impl<'a> ElfHeaderReader<'a, u32> {
    pub fn program_headers(
        &self,
    ) -> Result<
        impl Iterator<Item = ProgramHeaderReader<'a, u32, PhantomData<()>, u32>>,
        ElfReadError,
    > {
        self.program_headers_inner()
    }

    pub fn relocation_table(
        &self,
    ) -> Result<Option<&'a [RelocationTableEntry<u32>]>, ElfReadError> {
        self.relocation_table_inner::<PhantomData<()>, u32>()
    }
}

impl<'a> ElfHeaderReader<'a, u64> {
    pub fn program_headers(
        &self,
    ) -> Result<
        impl Iterator<Item = ProgramHeaderReader<'a, u64, u32, PhantomData<()>>>,
        ElfReadError,
    > {
        self.program_headers_inner()
    }

    pub fn relocation_table(
        &self,
    ) -> Result<Option<&'a [RelocationTableEntry<u64>]>, ElfReadError> {
        self.relocation_table_inner::<u32, PhantomData<()>>()
    }
}

struct SectionHeaderIter<'a, P> {
    raw_data: &'a [u8],
    offset: usize,
    count: u16,
    current: u16,
    size: u16,
    _phantom: PhantomData<P>,
}

impl<'a, P: Copy + TryInto<usize>> Iterator for SectionHeaderIter<'a, P> {
    type Item = SectionHeaderReader<'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }

        let entry_offset = self.offset + self.current as usize * self.size as usize;
        self.current += 1;

        Some(SectionHeaderReader::new(self.raw_data, entry_offset).unwrap())
    }
}

struct ProgramHeaderIter<'a, P, F1, F2> {
    raw_data: &'a [u8],
    offset: usize,
    count: u16,
    current: u16,
    size: u16,
    _phantom: PhantomData<(P, F1, F2)>,
}

impl<'a, P: Copy + TryInto<usize>, F1: Copy, F2: Copy> Iterator
    for ProgramHeaderIter<'a, P, F1, F2>
{
    type Item = ProgramHeaderReader<'a, P, F1, F2>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }

        let entry_offset = self.offset + self.current as usize * self.size as usize;
        self.current += 1;

        Some(ProgramHeaderReader::new(self.raw_data, entry_offset).unwrap())
    }
}
