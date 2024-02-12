#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElfHeader<P> {
    pub kind: ObjectKind,
    pub arch: Arch,
    pub version: ElfVersion,
    pub entrypoint: P,
    pub program_header_ptr: P,
    pub section_header_ptr: P,
    pub arch_flags: u32,
    pub header_size: u16,
    pub program_header_table_entry_size: u16,
    pub program_header_table_len: u16,
    pub section_header_table_entry_size: u16,
    pub section_header_table_len: u16,
    pub section_header_name_index: u16,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectKind {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    SharedObject = 3,
    Other,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arch {
    Archx86 = 0x03,
    Archx86_64 = 0x3E,
    Other,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElfVersion {
    Original = 1,
    Other,
}
