#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    start: u64,
    size: u64,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BootInfo<'a> {
    physical_memory_offset: u64,
    pre_kernel: MemoryRegion,
    kernel_code: MemoryRegion,
    kernel_stack: MemoryRegion,
    usable_memory: &'a [MemoryRegion],

    kernel_arguments: Option<&'a str>,
    bootloader_name: Option<&'a str>,
}
