#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BootInfo<'a> {
    pub physical_memory_offset: u64,

    pub stack_size: u64,

    pub kernel_code: MemoryRegion,
    pub bump_memory: MemoryRegion,

    pub usable_heap: MemoryRegion,
    pub usable_memory: &'a [MemoryRegion],

    pub kernel_arguments: Option<&'a str>,
    pub bootloader_name: Option<&'a str>,
}
