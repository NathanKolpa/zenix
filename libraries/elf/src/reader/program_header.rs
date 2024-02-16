use core::{
    marker::PhantomData,
    mem::{align_of, size_of},
};

use crate::{structure::program_header::*, ElfReadError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SectionKind {
    Null,
    Load,
    Dynamic,
    Intererp,
    Note,
    ProgramHeader,
    ThreadLocalStorage,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SectionFlags {
    flags: u32,
}

impl SectionFlags {
    const EXEC_BIT: u32 = 1 << 1;
    const WRITABLE_BIT: u32 = 2 << 1;
    const READABLE_BIT: u32 = 3 << 1;

    pub fn executable(&self) -> bool {
        (self.flags & Self::EXEC_BIT) != 0
    }

    pub fn readable(&self) -> bool {
        (self.flags & Self::READABLE_BIT) != 0
    }

    pub fn writable(&self) -> bool {
        (self.flags & Self::WRITABLE_BIT) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DynamicTag {
    Rela,
    RelaSize,
    RelaEnt,
    Other,
}

impl From<u32> for DynamicTag {
    fn from(value: u32) -> Self {
        match value {
            7 => Self::Rela,
            8 => Self::RelaSize,
            9 => Self::RelaEnt,
            _ => Self::Other,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct DynamicEntry<P> {
    tag: P,
    val: P,
}

impl<P: TryInto<u32> + Copy> DynamicEntry<P> {
    pub fn tag(&self) -> DynamicTag {
        let tag: Result<u32, _> = self.tag.try_into();

        match tag {
            Ok(num) => num.into(),
            Err(_) => DynamicTag::Other,
        }
    }
}

pub enum SegmentData<'a, P> {
    Bytes(&'a [u8]),
    Dynamic(DynamicSection<'a, P>),
}

pub struct DynamicSection<'a, P> {
    data: &'a [DynamicEntry<P>],
}

impl<'a, P: Copy + TryInto<u32>> DynamicSection<'a, P> {
    pub fn entries(&self) -> &'a [DynamicEntry<P>] {
        self.data
    }

    /// Returns a tuple with the following contents (in order):
    /// - `DT_RELA` This element holds the address of a relocation table.
    /// - `DT_RELASZ` This element holds the total size, in bytes, of the `DT_RELA` relocation table.
    /// - `DT_RELAENT` This element holds the size, in bytes, of the `DT_RELA` relocation entry.
    pub fn relocation_table_entries(&self) -> Option<(P, P, P)> {
        let mut rela = None;
        let mut relasz = None;
        let mut relaent = None;

        for entry in self.data {
            match entry.tag() {
                DynamicTag::Rela => rela = Some(entry.val),
                DynamicTag::RelaSize => relasz = Some(entry.val),
                DynamicTag::RelaEnt => relaent = Some(entry.val),
                _ => {}
            }
        }

        Some((rela?, relasz?, relaent?))
    }
}

#[derive(Clone)]
pub struct ProgramHeaderReader<'a, P, F1, F2> {
    raw_data: &'a [u8],
    offset: usize,
    header: ProgramHeader<P, F1, F2>,
}

impl<'a, P: Copy + TryInto<usize>, F1: Copy, F2: Copy> ProgramHeaderReader<'a, P, F1, F2> {
    pub fn new(raw_data: &'a [u8], offset: usize) -> Result<Self, ElfReadError> {
        let header = unsafe { super::read_struct(raw_data, offset) }?;
        Ok(Self {
            raw_data,
            header,
            offset,
        })
    }

    pub fn kind(&self) -> SectionKind {
        match self.header.kind {
            0 => SectionKind::Null,
            1 => SectionKind::Load,
            2 => SectionKind::Dynamic,
            3 => SectionKind::Intererp,
            4 => SectionKind::Note,
            6 => SectionKind::ProgramHeader,
            7 => SectionKind::ThreadLocalStorage,
            _ => SectionKind::Other,
        }
    }

    pub fn addr(&self) -> P {
        self.header.addr
    }

    pub fn memory_size(&self) -> P {
        self.header.memsize
    }

    pub fn file_size(&self) -> P {
        self.header.filesize
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn data_offset(&self) -> P {
        self.header.offset
    }

    pub fn data(&self) -> Result<SegmentData<'a, P>, ElfReadError> {
        match self.kind() {
            SectionKind::Dynamic => self.dynamic_data().map(SegmentData::Dynamic),
            _ => self.bytes().map(SegmentData::Bytes),
        }
    }

    fn dynamic_data(&self) -> Result<DynamicSection<'a, P>, ElfReadError> {
        debug_assert_eq!(SectionKind::Dynamic, self.kind());

        let bytes = self.bytes()?;

        if bytes.as_ptr() as usize % align_of::<DynamicEntry<P>>() != 0 {
            return Err(ElfReadError::NotAligned);
        }

        if bytes.len() % size_of::<DynamicEntry<P>>() != 0 {
            return Err(ElfReadError::NotAligned);
        }

        let len = bytes.len() / size_of::<DynamicEntry<P>>();

        let data = unsafe { core::slice::from_raw_parts(bytes.as_ptr() as *const _, len) };
        Ok(DynamicSection { data })
    }

    pub fn bytes(&self) -> Result<&'a [u8], ElfReadError> {
        let offset = self
            .header
            .offset
            .try_into()
            .map_err(|_| ElfReadError::TooSmall)?;

        let len = self
            .header
            .filesize
            .try_into()
            .map_err(|_| ElfReadError::TooSmall)?;

        let end = offset + len;

        Ok(&self.raw_data[offset..end])
    }
}

impl<'a, P: Copy + TryInto<usize>> ProgramHeaderReader<'a, P, u32, PhantomData<()>> {
    pub fn flags(&self) -> SectionFlags {
        SectionFlags {
            flags: self.header.flags_64,
        }
    }
}

impl<'a, P: Copy + TryInto<usize>> ProgramHeaderReader<'a, P, PhantomData<()>, u32> {
    pub fn flags(&self) -> SectionFlags {
        SectionFlags {
            flags: self.header.flags_32,
        }
    }
}

pub type ProgramHeaderReader64<'a> = ProgramHeaderReader<'a, u64, u32, PhantomData<()>>;
