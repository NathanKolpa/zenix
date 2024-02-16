use crate::{structure::segment_header::SectionHeader, ElfReadError};

#[allow(dead_code)] // TODO: use this struct
pub struct SectionHeaderReader<'a, P> {
    raw_data: &'a [u8],
    header: SectionHeader<P>,
}

impl<'a, P: Clone + TryInto<usize>> SectionHeaderReader<'a, P> {
    pub fn new(raw_data: &'a [u8], offset: usize) -> Result<Self, ElfReadError> {
        let header = unsafe { super::read_struct(raw_data, offset) }?;
        Ok(Self { raw_data, header })
    }
}
