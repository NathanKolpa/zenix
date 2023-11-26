use core::alloc::{GlobalAlloc, Layout};

const HEAP_SIZE: usize = 1024 * 1024;
static mut BACKING: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub struct KernelAlloc<'a> {
    backing: &'a [u8],
}

impl<'a> KernelAlloc<'a> {
    pub const fn new(backing: &'a [u8]) -> Self {
        Self { backing }
    }
}

unsafe impl GlobalAlloc for KernelAlloc<'_> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.backing.as_ptr() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}

#[global_allocator]
pub static KERNEL_ALLOC: KernelAlloc = KernelAlloc::new(unsafe { &mut BACKING });
