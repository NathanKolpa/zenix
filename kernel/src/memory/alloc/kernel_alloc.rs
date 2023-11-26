use core::alloc::{GlobalAlloc, Layout};
use core::fmt::{Debug, Formatter};
use core::mem::{align_of, size_of, MaybeUninit};

use crate::util::address::VirtualAddress;
use crate::util::spin::SpinLock;

const HEAP_SIZE: usize = 1024 * 1024;
static mut BACKING: [MaybeUninit<u64>; HEAP_SIZE / 8] = [MaybeUninit::uninit(); HEAP_SIZE / 8]; // we must align the backing with the FreeNode

static BACKING_REF: SpinLock<Option<&'static mut [MaybeUninit<u64>]>> =
    SpinLock::new(Some(unsafe { &mut BACKING }));

#[global_allocator]
pub static KERNEL_ALLOC: KernelAlloc = KernelAlloc::new();

/// Add backing to the kernel heap.
///
/// Calling this function more than once will not do anything.
pub fn init_heap() -> usize {
    let mut backing_ref = BACKING_REF.lock();

    if let Some(backing_ref) = backing_ref.take() {
        KERNEL_ALLOC.add_backing(backing_ref);
        HEAP_SIZE
    } else {
        0
    }
}

struct FreeNode<'a> {
    size: usize,
    next: Option<&'a mut FreeNode<'a>>,
}

impl Debug for FreeNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FreeNode")
            .field("start", &(self.node_start_addr() as *const Self))
            .field("end", &(self.end_addr() as *const Self))
            .field("size", &self.size)
            .field("next", &self.next)
            .finish()
    }
}

impl<'a> FreeNode<'a> {
    fn node_start_addr(&self) -> usize {
        (self as *const Self) as usize
    }

    fn end_addr(&self) -> usize {
        self.node_start_addr() + self.size
    }

    fn fit_layout(&mut self, layout: Layout) -> Option<usize> {
        let alloc_start = VirtualAddress::align_ptr_up(self.node_start_addr(), layout.align());
        let alloc_end = alloc_start.checked_add(layout.size())?;

        if alloc_end > self.end_addr() {
            // region too small
            return None;
        }

        let excess_size = self.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < size_of::<Self>() {
            // rest of region too small to hold a ListNode (required because the
            // allocation splits the region in a used and a free part)
            return None;
        }

        // region suitable for allocation
        Some(alloc_start)
    }

    fn allocate(
        current_slot: &mut Option<&'a mut Self>,
        layout: Layout,
    ) -> Option<(&'a mut Self, usize)> {
        let Some(current) = current_slot else {
            return None;
        };

        if let Some(ptr) = current.fit_layout(layout) {
            let next = current.next.take();
            let ret = Some((current_slot.take().unwrap(), ptr));
            *current_slot = next;
            return ret;
        }

        Self::allocate(&mut current.next, layout)
    }

    fn defrag(&mut self) {
        let end = self.end_addr();
        let Some(next) = self.next.as_mut() else {
            return;
        };

        if end == next.node_start_addr() {
            self.size += next.size;
            self.next = next.next.take();
        }
    }
}

#[derive(Debug)]
pub struct KernelAlloc<'a> {
    head: SpinLock<Option<&'a mut FreeNode<'a>>>,
}

impl<'a> KernelAlloc<'a> {
    pub const fn new() -> Self {
        Self {
            head: SpinLock::new(None),
        }
    }

    unsafe fn add_free_region(&self, addr: usize, size: usize) {
        let mut current_head = self.head.lock();

        let head = unsafe { &mut *(addr as *mut FreeNode) };
        *head = FreeNode {
            next: current_head.take(),
            size,
        };

        head.defrag();

        *current_head = Some(head);
    }

    pub fn add_backing(&self, backing: &'a mut [MaybeUninit<u64>]) {
        assert!(backing.len() >= size_of::<FreeNode>());

        let addr = backing.as_mut_ptr() as usize;
        assert_eq!(
            VirtualAddress::align_ptr_up(addr, align_of::<FreeNode>()),
            addr
        );

        unsafe { self.add_free_region(addr, core::mem::size_of_val(backing)) }
    }

    fn size_align(layout: Layout) -> Layout {
        let layout = layout
            .align_to(align_of::<FreeNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(size_of::<FreeNode>());
        Layout::from_size_align(size, layout.align()).unwrap()
    }
}

unsafe impl GlobalAlloc for KernelAlloc<'_> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout = Self::size_align(layout);
        let mut head = self.head.lock();

        FreeNode::allocate(&mut head, layout)
            .map(|(node, ptr)| {
                let alloc_end = ptr.checked_add(layout.size()).expect("overflow");
                let excess_size = node.end_addr() - alloc_end;
                if excess_size > 0 {
                    drop(head);
                    self.add_free_region(alloc_end, excess_size);
                }

                ptr as *mut u8
            })
            .unwrap_or(core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout = Self::size_align(layout);
        self.add_free_region(ptr as usize, layout.size());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    #[test_case]
    fn test_reuse_ptr() {
        let mut backing = [MaybeUninit::uninit(); 64];
        let alloc = KernelAlloc::new();
        alloc.add_backing(&mut backing);

        let layout = Layout::new::<u64>();

        unsafe {
            let ptr = alloc.alloc(layout);
            alloc.dealloc(ptr, layout);
            assert_eq!(ptr, alloc.alloc(layout));
        };
    }

    #[test_case]
    fn test_two_boxes_not_colliding() {
        let value_1 = Box::new(0xF0F0);
        let value_2: Box<u64> = Box::new(0xdeadbeef);

        assert_eq!(*value_1, 0xF0F0);
        assert_eq!(*value_2, 0xdeadbeef);
    }

    #[test_case]
    fn test_reuse_more_then_the_entire_heap() {
        for i in 0..HEAP_SIZE {
            let value = Box::new(i);
            assert_eq!(*value, i);
        }
    }

    #[test_case]
    fn test_pushing_to_large_vec() {
        let count = 1000;
        let mut vec = Vec::new();

        for i in 0..count {
            vec.push(i);
        }

        assert_eq!(vec.iter().sum::<u64>(), (count - 1) * count / 2);
    }
}
