#![no_std]

use essentials::address::PhysicalAddress;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub start: PhysicalAddress,
    pub size: usize,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BootInfo {
    pub physical_memory_offset: usize,
    pub pre_kernel: MemoryRegion,
    pub kernel_code: MemoryRegion,
    pub kernel_stack: MemoryRegion,
    pub usable_memory: &'static [MemoryRegion],

    pub kernel_arguments: Option<&'static str>,
    pub bootloader_name: Option<&'static str>,
}
