use crate::{reader::ReadError, structure::program_header::*};

pub use crate::structure::program_header::SegmentKind;

pub struct ProgramHeaderReader<'a, P> {
    raw_data: &'a [u8],
    header: ProgramHeader<P>,
}
impl<'a, P: Copy> ProgramHeaderReader<'a, P> {
    pub unsafe fn new_unchecked(raw_data: &'a [u8], offset: usize) -> Self {
        let header_slice = &raw_data[offset..];
        let header = *(header_slice.as_ptr() as *const ProgramHeader<P>);
        Self { raw_data, header }
    }
}
