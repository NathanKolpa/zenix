use alloc::vec::Vec;
use core::cmp::max;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{memory::alloc::frame_alloc::level::Level, utils::InterruptGuard};
use essentials::address::PhysicalAddress;
use essentials::spin::SpinLock;

use super::MIN_ORDER;

pub struct Zone {
    addr_start: PhysicalAddress,
    addr_end: PhysicalAddress,
    physical_memory_offset: usize,

    available: AtomicUsize,
    levels: Vec<InterruptGuard<SpinLock<Level>>>, // TODO: add this to the eternal alloc
}

impl Zone {
    pub unsafe fn new(
        addr_start: PhysicalAddress,
        addr_end: PhysicalAddress,
        physical_memory_offset: usize,
    ) -> Self {
        assert!(addr_end > addr_start);
        assert!((addr_end - addr_start).as_usize() >= 2usize.pow(MIN_ORDER as u32));

        let total_size = (addr_end - addr_start).as_usize();
        let largest_order = order_of_two(total_size.next_power_of_two());

        assert!(largest_order >= MIN_ORDER);

        let level_count = (largest_order - MIN_ORDER) as usize;

        let mut levels = Vec::with_capacity(level_count + 1);

        for order in MIN_ORDER..=largest_order {
            levels.push(InterruptGuard::new_lock(Level::new(
                order,
                largest_order,
                addr_start,
                physical_memory_offset,
            )))
        }

        Self::distribute_unused_memory(addr_start, total_size, largest_order, &mut levels);

        Self {
            physical_memory_offset,
            addr_start,
            addr_end,
            levels,
            available: AtomicUsize::new(total_size),
        }
    }

    fn level_allocate(&self, index: usize) -> Option<(PhysicalAddress, usize)> {
        let level = self.levels.get(index)?;

        let level_lock = level.guard();
        let mut level_lock = level_lock.lock();

        level_lock
            .pop_from_list_and_mark_as_used()
            .map(|addr| (addr, index))
            .or_else(|| {
                self.level_allocate(index + 1)
                    .map(|(allocated_addr, parent_index)| {
                        let target_index = parent_index - 1;
                        assert_eq!(target_index, index);

                        let first_half = allocated_addr;
                        let second_half = allocated_addr
                            + 2usize.pow(Self::level_index_to_order(target_index) as u32);

                        unsafe { level_lock.push_free_block(second_half) };
                        level_lock.mark_as_used(first_half);

                        (first_half, target_index)
                    })
            })
    }

    unsafe fn level_deallocate(&self, index: usize, addr: PhysicalAddress) {
        if index >= self.levels.len() {
            return;
        }

        // holding on to all the locks is important.
        // we dont want to coalesce a block when another thread tries allocate.
        let level_lock = self.levels[index].guard();
        let mut level_lock = level_lock.lock();

        let aligned_addr = addr.align_down(level_lock.block_size());

        level_lock.mark_as_unused(aligned_addr);

        if level_lock.has_buddy(aligned_addr) && !level_lock.is_buddy_used(aligned_addr) {
            level_lock.remove_buddy_from_list(aligned_addr);
            self.level_deallocate(index + 1, aligned_addr);
        } else {
            level_lock.push_free_block(aligned_addr);
        }
    }

    pub fn allocate_zeroed(&self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let (addr, size) = self.allocate(size)?;

        let bytes = unsafe {
            core::slice::from_raw_parts_mut(
                (addr.as_usize() + self.physical_memory_offset) as *mut u8,
                size,
            )
        };

        for byte in bytes {
            *byte = 0;
        }

        Some((addr, size))
    }

    pub fn allocate(&self, size: usize) -> Option<(PhysicalAddress, usize)> {
        assert!(size.is_power_of_two());

        let order = max(order_of_two(size), MIN_ORDER);
        let level_index = Self::order_to_level_index(order);

        self.level_allocate(level_index).map(|(addr, index)| {
            let size = 2usize.pow(Self::level_index_to_order(index) as u32);
            self.available.fetch_sub(size, Ordering::SeqCst);
            (addr, size)
        })
    }

    pub unsafe fn deallocate(&self, addr: PhysicalAddress) {
        assert!(self.contains(addr));
        let level_index = self.find_allocation_level_index(addr);
        self.level_deallocate(level_index, addr);
    }

    fn find_allocation_level_index(&self, addr: PhysicalAddress) -> usize {
        for (index, level) in self.levels.iter().enumerate() {
            let block_size = 2usize.pow(Self::level_index_to_order(index) as u32);

            let aligned_addr = addr.align_down(block_size);

            let level_lock = level.guard();
            let level_lock = level_lock.lock();

            if level_lock.is_within_allocated_block(aligned_addr) {
                return index;
            }
        }

        panic!("Could not determine the allocation size on free because the address is not aligned with a used block")
    }

    unsafe fn distribute_unused_memory(
        start: PhysicalAddress,
        memory_left: usize,
        order: u8,
        levels: &mut [InterruptGuard<SpinLock<Level>>],
    ) {
        if order < MIN_ORDER || memory_left == 0 {
            return;
        }

        let order_size = 2usize.pow(order as u32);

        let next_order = order - 1;

        let level = &mut levels[Self::order_to_level_index(order)];
        let level_lock = level.as_mut().as_mut();

        // a level needs exactly the order_size no less, if we can't satisfy that we should skip.
        if order_size > memory_left {
            // when there is still memory left to place,
            // then the current level should be marked as used.
            if memory_left >= 2usize.pow(MIN_ORDER as u32) {
                level_lock.mark_as_used(start);
                Self::distribute_unused_memory(start, memory_left, next_order, levels);
            }

            return;
        }

        level_lock.push_free_block(start);

        if level_lock.has_buddy(start) {
            level_lock.mark_buddy_as_used(start);
        }

        Self::distribute_unused_memory(
            start + order_size,
            memory_left - order_size,
            next_order,
            levels,
        );
    }

    fn order_to_level_index(order: u8) -> usize {
        (order - MIN_ORDER) as usize
    }

    fn level_index_to_order(index: usize) -> u8 {
        index as u8 + MIN_ORDER
    }

    pub fn size(&self) -> usize {
        (self.addr_end - self.addr_start).as_usize()
    }

    pub fn available(&self) -> usize {
        self.available.load(Ordering::Relaxed)
    }

    pub fn contains(&self, addr: PhysicalAddress) -> bool {
        addr >= self.addr_start && addr <= self.addr_end
    }

    pub fn clashes(&self, addr: PhysicalAddress, size: usize) -> bool {
        self.contains(addr)
            || self.contains(addr + size)
            || self.addr_start >= addr && self.addr_start <= addr + size
    }
}

fn order_of_two(size: usize) -> u8 {
    assert!(size.is_power_of_two());

    let mut result = 0u8;
    let mut value = size;

    while value > 1 {
        value >>= 1;
        result += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ceil_order_of_two() {
        assert_eq!(order_of_two(2), 1);
        assert_eq!(order_of_two(4), 2);
        assert_eq!(order_of_two(16), 4);
    }
}
