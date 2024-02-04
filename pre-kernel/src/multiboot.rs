use core::{marker::PhantomData, mem::size_of, u8, usize};

#[repr(C)]
pub struct MultibootMMapEntry {
    entry_size: u32,
    addr: u64,
    len: u64,
    kind: u32,
}

impl MultibootMMapEntry {
    pub fn addr(&self) -> u64 {
        self.addr
    }

    pub fn len(&self) -> u64 {
        self.len
    }

    pub fn kind(&self) -> u32 {
        self.kind
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

    pub fn mmap(&self) -> Option<impl Iterator<Item = &MultibootMMapEntry> + '_ + Clone> {
        if self.flags & (1 << 6) == 0 {
            return None;
        }

        Some(MemoryMapIter {
            addr: self.mmap_addr as *const _,
            info: self,
        })
    }
}

#[derive(Clone)]
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
