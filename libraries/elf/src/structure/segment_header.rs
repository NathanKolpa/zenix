#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SectionHeader<P> {
    name_offset: u32,
    kind: SectionKind,
    flags: P,
    addr: P,
    offset: P,
    size: P,
    link: u32,
    info: u32,
    addr_align: P,
    entry_size: P,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SectionKind {
    Null,
}
