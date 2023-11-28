use alloc::vec;
use alloc::vec::Vec;

use crate::memory::alloc::frame_alloc::node::FreeListNode;
use crate::util::address::{PhysicalAddress, VirtualAddress};
use crate::util::Bitmap;

pub struct Level {
    free_list: Option<PhysicalAddress>,
    order: u8,
    bitmap: Bitmap<Vec<u8>>, // TODO: add this to the eternal alloc
    base: PhysicalAddress,
    global_offset: usize,
}

impl Level {
    pub unsafe fn new(
        order: u8,
        largest_order: u8,
        base: PhysicalAddress,
        global_offset: usize,
    ) -> Self {
        let bits = 2usize.pow(((largest_order - order) as usize) as u32);
        let bytes = (bits + 8 - 1) / 8; // div ceil 8 the number of bits

        Self {
            free_list: None,
            order,
            bitmap: Bitmap::new(vec![0; bytes]),
            global_offset,
            base,
        }
    }

    pub fn size(&self) -> usize {
        2usize.pow(self.order as u32)
    }

    pub unsafe fn add_free_block(&mut self, addr: PhysicalAddress) {
        self.add_to_free_list(addr);
    }

    pub fn mark_as_used(&mut self, addr: PhysicalAddress) {
        debug_println!("Bit: {}", self.bit_in_bitmap(addr));
        self.bitmap.set(self.bit_in_bitmap(addr));
    }

    pub fn mark_buddy_as_used(&mut self, addr: PhysicalAddress) {
        debug_println!("Bit: {}", self.buddy_bit(self.bit_in_bitmap(addr)));
        self.bitmap.set(self.buddy_bit(self.bit_in_bitmap(addr)));
    }

    fn bit_in_bitmap(&self, addr: PhysicalAddress) -> usize {
        let diff = addr - self.base;
        diff.as_usize() / self.size()
    }

    fn buddy_bit(&self, bit: usize) -> usize {
        if bit % 2 == 0 {
            bit + 1
        } else {
            bit - 1
        }
    }

    unsafe fn add_to_free_list(&mut self, addr: PhysicalAddress) {
        let new_node = self.node_ptr_as_ref(addr);
        new_node.next = self.free_list.take();
        self.free_list = Some(addr);
    }

    unsafe fn node_ptr_as_ref(&self, addr: PhysicalAddress) -> &'static mut FreeListNode {
        &mut *(self.phys_to_virt(addr).as_mut_ptr())
    }

    fn phys_to_virt(&self, phys: PhysicalAddress) -> VirtualAddress {
        VirtualAddress::from(phys.as_usize() + self.global_offset)
    }
}
