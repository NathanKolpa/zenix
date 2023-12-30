use alloc::{vec, vec::Vec};

use crate::util::address::{PhysicalAddress, VirtualAddress};
use crate::util::Bitmap;

struct Node {
    next: PhysicalAddress,
    prev: PhysicalAddress,
}

pub struct Level {
    head: PhysicalAddress,
    tail: PhysicalAddress,
    order: u8,
    bitmap: Bitmap<Vec<u8>>, // TODO: add this to the eternal alloc
    base: PhysicalAddress,
    global_offset: usize,
    total_blocks: usize,
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
            head: PhysicalAddress::null(),
            tail: PhysicalAddress::null(),
            order,
            bitmap: Bitmap::new(vec![0; bytes]),
            global_offset,
            base,
            total_blocks: bits,
        }
    }

    pub fn block_size(&self) -> usize {
        2usize.pow(self.order as u32)
    }

    pub fn is_within_allocated_block(&self, addr: PhysicalAddress) -> bool {
        let addr = addr.align_down(self.block_size());
        let bit = self.bit_in_bitmap(addr);
        self.bitmap.contains(bit)
    }

    pub unsafe fn push_free_block(&mut self, new_node: PhysicalAddress) {
        let new_node_ref = &mut *self.phys_to_virt(new_node).as_mut_ptr::<Node>();

        if self.head.is_null() {
            new_node_ref.next = PhysicalAddress::null();
            new_node_ref.prev = PhysicalAddress::null();
            self.head = new_node;
            self.tail = new_node;
            return;
        }

        self.insert_before(self.head, new_node)
    }

    unsafe fn insert_before(&mut self, node: PhysicalAddress, new_node: PhysicalAddress) {
        let node_ref = &mut *self.phys_to_virt(node).as_mut_ptr::<Node>();
        let new_node_ref = &mut *self.phys_to_virt(new_node).as_mut_ptr::<Node>();

        new_node_ref.next = node;

        if node_ref.prev.is_null() {
            new_node_ref.prev = PhysicalAddress::null();
            self.head = new_node;
        } else {
            new_node_ref.prev = node_ref.prev;
            let node_prev_ref = &mut *self.phys_to_virt(node_ref.prev).as_mut_ptr::<Node>();
            node_prev_ref.next = new_node;
        }

        node_ref.prev = new_node;
    }

    pub fn pop_from_list_and_mark_as_used(&mut self) -> Option<PhysicalAddress> {
        let block = self.pop_free_block()?;
        let block_bit = self.bit_in_bitmap(block);

        debug_assert!(!self.bitmap.contains(block_bit));
        self.bitmap.set(block_bit);

        Some(block)
    }

    fn pop_free_block(&mut self) -> Option<PhysicalAddress> {
        if self.head.is_null() {
            return None;
        }

        let node = self.head;
        unsafe { self.remove_block(node) };
        Some(node)
    }

    unsafe fn remove_block(&mut self, node: PhysicalAddress) {
        let node_ref = &mut *self.phys_to_virt(node).as_mut_ptr::<Node>();

        if node_ref.prev.is_null() {
            self.head = node_ref.next;
        } else {
            let node_prev_ref = &mut *self.phys_to_virt(node_ref.prev).as_mut_ptr::<Node>();
            node_prev_ref.next = node_ref.next;
        }

        if node_ref.next.is_null() {
            self.tail = node_ref.prev;
        } else {
            let node_next_ref = &mut *self.phys_to_virt(node_ref.next).as_mut_ptr::<Node>();
            node_next_ref.prev = node_ref.prev;
        }
    }

    pub fn mark_as_used(&mut self, addr: PhysicalAddress) {
        self.bitmap.set(self.bit_in_bitmap(addr));
    }

    pub fn mark_as_unused(&mut self, addr: PhysicalAddress) {
        self.bitmap.clear(self.bit_in_bitmap(addr));
    }

    pub unsafe fn remove_buddy_from_list(&mut self, addr: PhysicalAddress) {
        self.remove_block(self.buddy_address(addr));
    }

    pub fn buddy_address(&self, addr: PhysicalAddress) -> PhysicalAddress {
        self.bit_to_addr(self.buddy_bit(self.bit_in_bitmap(addr)))
    }

    pub fn is_buddy_used(&self, addr: PhysicalAddress) -> bool {
        self.bitmap
            .contains(self.buddy_bit(self.bit_in_bitmap(addr)))
    }

    pub fn mark_buddy_as_used(&mut self, addr: PhysicalAddress) {
        self.bitmap.set(self.buddy_bit(self.bit_in_bitmap(addr)));
    }

    pub fn has_buddy(&self, addr: PhysicalAddress) -> bool {
        let bit = self.buddy_bit(self.bit_in_bitmap(addr));
        bit < self.total_blocks
    }

    fn bit_to_addr(&self, bit: usize) -> PhysicalAddress {
        self.base + bit * self.block_size()
    }

    fn bit_in_bitmap(&self, addr: PhysicalAddress) -> usize {
        let diff = addr - self.base;
        diff.as_usize() / self.block_size()
    }

    fn buddy_bit(&self, bit: usize) -> usize {
        if bit % 2 == 0 {
            bit + 1
        } else {
            bit - 1
        }
    }

    fn phys_to_virt(&self, phys: PhysicalAddress) -> VirtualAddress {
        VirtualAddress::from(phys.as_usize() + self.global_offset)
    }
}
