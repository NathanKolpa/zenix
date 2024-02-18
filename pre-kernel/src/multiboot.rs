use core::mem::size_of;

use bootinfo::MemoryRegion;

pub const MULTIBOOT_MAGIC: u32 = 0x2BADB002;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MultibootModule {
    start: u32,
    end: u32,
    cmdline: u32,
    _reserved: u32,
}

impl MultibootModule {
    pub fn addr(&self) -> u32 {
        self.start
    }

    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn as_info_region(&self) -> MemoryRegion {
        MemoryRegion {
            start: self.start as u64,
            size: (self.end - self.start) as u64,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootMMapEntry {
    entry_size: u32,
    addr: u64,
    size: u64,
    kind: u32,
}

impl MultibootMMapEntry {
    pub fn addr(&self) -> u64 {
        self.addr
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn kind(&self) -> u32 {
        self.kind
    }

    pub fn is_usable(&self) -> bool {
        self.kind == 1
    }
}

#[repr(C)]
pub struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
    mods_len: u32,
    mods_addr: u32,

    _syms: [u8; 16],

    mmap_len: u32,
    mmap_addr: u32,

    drivers_len: u32,
    drivers_addr: u32,
    config_table: u32,

    boot_loader_name: u32,
}

impl MultibootInfo {
    unsafe fn str_len(mut addr: *const u8) -> usize {
        let mut len = 0;

        loop {
            if *addr == 0 {
                break;
            }

            len += 1;
            addr = addr.add(1);
        }

        len
    }

    pub fn cmdline(&self) -> Option<&str> {
        if self.flags & (1 << 2) == 0 {
            return None;
        }

        let buff = unsafe {
            let len = Self::str_len(self.cmdline as *const _);
            core::slice::from_raw_parts(self.cmdline as *const u8, len)
        };

        core::str::from_utf8(buff).ok()
    }

    pub fn boot_loader_name(&self) -> Option<&str> {
        if self.flags & (1 << 9) == 0 {
            return None;
        }

        let buff = unsafe {
            let len = Self::str_len(self.boot_loader_name as *const _);
            core::slice::from_raw_parts(self.boot_loader_name as *const u8, len)
        };

        core::str::from_utf8(buff).ok()
    }

    pub fn mmap(&self) -> Option<impl Iterator<Item = &MultibootMMapEntry> + '_ + Copy> {
        if self.flags & (1 << 6) == 0 {
            return None;
        }

        Some(MemoryMapIter {
            addr: self.mmap_addr as *const _,
            info: self,
        })
    }

    pub fn take_first_mod(&mut self) -> Option<MultibootModule> {
        if self.flags & (1 << 3) == 0 {
            return None;
        }

        if self.mods_len == 0 {
            return None;
        }

        self.mods_len -= 1;
        let addr = self.mods_addr;
        self.mods_addr += size_of::<MultibootModule>() as u32;

        Some(unsafe { *(addr as *mut MultibootModule) })
    }

    pub fn mods(&self) -> Option<&[MultibootModule]> {
        Some(unsafe {
            core::slice::from_raw_parts(self.mods_addr as *const _, self.mods_len as usize)
        })
    }
}

#[derive(Clone, Copy)]
struct MemoryMapIter<'a> {
    addr: *const MultibootMMapEntry,
    info: &'a MultibootInfo,
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MultibootMMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.addr as usize >= self.info.mmap_addr as usize + self.info.mmap_len as usize {
            return None;
        }

        let entry = unsafe { &*self.addr };
        unsafe {
            self.addr = self
                .addr
                .byte_add(entry.entry_size as usize + size_of::<u32>());
        }

        Some(entry)
    }
}
