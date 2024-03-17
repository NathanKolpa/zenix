#![no_std]

use core::fmt::Debug;

use essentials::address::VirtualAddress;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
}

impl MemoryRegion {
    pub fn merge_with(&self, rhs: Self) -> Option<Self> {
        let self_end = self.start + self.size;

        if self_end != rhs.start {
            return None;
        }

        Some(Self {
            start: self.start,
            size: self.size + rhs.size,
        })
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BootInfoData {
    pub physical_memory_offset: u64,

    pub kernel_stack: MemoryRegion,

    pub kernel_code: MemoryRegion,
    pub bump_memory: MemoryRegion,

    pub usable_heap: MemoryRegion,

    pub usable_memory_addr: u64,
    pub usable_memory_len: u64,

    pub kernel_arguments_addr: u64,
    pub kernel_arguments_len: u64,

    pub bootloader_name_addr: u64,
    pub bootloader_name_len: u64,

    pub rsdp_addr: u64,
}

pub struct BootInfo {
    data: &'static BootInfoData,
}

impl BootInfo {
    pub unsafe fn deref_ptr(info: *const BootInfoData) -> Self {
        let data = &*info;
        Self { data }
    }

    pub fn physycal_memory_offset(&self) -> usize {
        self.data.physical_memory_offset as usize
    }

    pub fn usable_memory(&self) -> &'static [MemoryRegion] {
        unsafe {
            core::slice::from_raw_parts(
                self.data.usable_memory_addr as *const _,
                self.data.usable_memory_len as usize,
            )
        }
    }

    pub fn kernel_stack(&self) -> MemoryRegion {
        self.data.kernel_stack
    }

    pub fn kernel_code(&self) -> MemoryRegion {
        self.data.kernel_code
    }

    pub fn bump_memory(&self) -> MemoryRegion {
        self.data.bump_memory
    }

    pub fn usable_heap(&self) -> MemoryRegion {
        self.data.usable_heap
    }

    pub fn rsdp_addr(&self) -> Option<VirtualAddress> {
        let addr = self.data.rsdp_addr;

        if addr == 0 {
            return None;
        }

        Some(addr.into())
    }

    pub fn kernel_arguments(&self) -> Option<&'static str> {
        if self.data.kernel_arguments_addr == 0 || self.data.kernel_arguments_len == 0 {
            return None;
        }

        Some(unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.data.kernel_arguments_addr as *const _,
                self.data.kernel_arguments_len as usize,
            ))
        })
    }

    pub fn bootloader_name(&self) -> Option<&'static str> {
        if self.data.bootloader_name_addr == 0 || self.data.bootloader_name_len == 0 {
            return None;
        }

        Some(unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.data.bootloader_name_addr as *const _,
                self.data.bootloader_name_len as usize,
            ))
        })
    }
}

impl Debug for BootInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BootInfo")
            .field("physycal_memory_offset", &self.physycal_memory_offset())
            .field("usable_memory", &self.usable_memory())
            .field("usable_heap", &self.usable_heap())
            .field("bump_memory", &self.bump_memory())
            .field("kernel_arguments", &self.kernel_arguments())
            .field("bootloader_name", &self.bootloader_name())
            .field("rsdp_addr", &self.rsdp_addr())
            .finish()
    }
}
