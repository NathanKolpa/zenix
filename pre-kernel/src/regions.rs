use crate::{
    rsdp_detection::{RSDP_ADDR_END, RSDP_ADDR_START},
    vga::{VGA_ADDR, VGA_LEN},
};

extern "C" {
    pub static BUMP_MEMORY_START: u8;
    pub static BUMP_MEMORY_END: u8;

    pub static PRE_KERNEL_START: u8;
    pub static PRE_KERNEL_END: u8;

    pub static STACK_START: u8;
    pub static STACK_END: u8;
}

#[derive(Clone, Debug)]
pub struct KnownRegion {
    pub start: u64,
    pub size: u64,
    pub executable: bool,
    pub writable: bool,
    pub mmio: bool,
}

pub fn known_regions() -> impl Iterator<Item = KnownRegion> + Clone {
    [
        KnownRegion {
            start: unsafe { &STACK_START as *const _ as u64 },
            size: unsafe { &STACK_END as *const _ as u64 - (&STACK_START as *const _ as u64) },
            executable: false,
            writable: true,
            mmio: false,
        },
        KnownRegion {
            start: unsafe { &PRE_KERNEL_START as *const _ as u64 },
            size: unsafe {
                &PRE_KERNEL_END as *const _ as u64 - (&PRE_KERNEL_START as *const _ as u64)
            },
            executable: true,
            writable: true,
            mmio: false,
        },
        KnownRegion {
            start: unsafe { &BUMP_MEMORY_START as *const _ as u64 },
            size: unsafe {
                &BUMP_MEMORY_END as *const _ as u64 - (&BUMP_MEMORY_START as *const _ as u64)
            },
            executable: false,
            writable: true,
            mmio: false,
        },
        KnownRegion {
            start: VGA_ADDR.as_u64(),
            size: VGA_LEN as u64,
            executable: false,
            writable: true,
            mmio: true,
        },
        KnownRegion {
            start: 0xFEE00000,
            size: 0xFFF,
            writable: true,
            executable: false,
            mmio: true,
        },
        KnownRegion {
            start: RSDP_ADDR_START as u64,
            size: (RSDP_ADDR_END - RSDP_ADDR_START) as u64,
            writable: false,
            executable: false,
            mmio: false,
        },
    ]
    .into_iter()
}

pub fn stack_size() -> u64 {
    unsafe { &STACK_END as *const _ as u64 - (&STACK_START as *const _ as u64) }
}
