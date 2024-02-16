#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElfHeader<P> {
    pub kind: u16,
    pub arch: u16,
    pub version: u32,
    pub entrypoint: P,
    pub program_header_ptr: P,
    pub section_header_offset: P,
    pub arch_flags: u32,
    pub header_size: u16,
    pub program_header_table_entry_size: u16,
    pub program_header_table_len: u16,
    pub section_header_table_entry_size: u16,
    pub section_header_table_len: u16,
    pub section_header_name_index: u16,
}
