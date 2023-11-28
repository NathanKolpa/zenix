use alloc::vec::Vec;

use crate::memory::alloc::frame_alloc::level::Level;
use crate::util::address::{PhysicalAddress, VirtualAddress};
use crate::util::display::ReadableSize;

use super::MIN_ORDER;

pub struct Zone {
    addr_start: PhysicalAddress,
    addr_end: PhysicalAddress,

    levels: Vec<Level>, // TODO: add this to the eternal alloc
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
            levels.push(Level::new(
                order,
                largest_order,
                addr_start,
                physical_memory_offset,
            ))
        }

        Self::distribute_unused_memory(addr_start, total_size, largest_order, &mut levels);

        Self {
            addr_start,
            addr_end,
            levels,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<(PhysicalAddress, usize)> {
        let size = size.next_power_of_two();
        let _order = order_of_two(size);

        todo!()
    }

    unsafe fn distribute_unused_memory(
        start: PhysicalAddress,
        memory_left: usize,
        order: u8,
        levels: &mut [Level],
    ) {
        if order < MIN_ORDER || memory_left == 0 {
            return;
        }

        let order_size = 2usize.pow(order as u32);

        debug_println!(
            "> order {order} ({}), we have {} memory left.",
            ReadableSize::new(order_size),
            ReadableSize::new(memory_left)
        );

        let next_order = order - 1;

        let level = &mut levels[Self::order_to_level_index(order)];

        // a level needs exactly the order_size no less, if we can't satisfy that we should skip.
        if order_size > memory_left {
            debug_println!("\tCan't satisfy the block size requirement.");
            // when there is still memory left to place, then the current level should be marked as used.
            if memory_left >= 2usize.pow(MIN_ORDER as u32) {
                debug_println!("\tBut there's still memory left so we mark this level as used and we keep going.");
                level.mark_as_used(start);
                Self::distribute_unused_memory(start, memory_left, next_order, levels);
            }

            return;
        }

        debug_println!("\tAdded added addr {start} to the level's free list, meaning the level has now {} of free memory", ReadableSize::new(order_size));
        level.add_free_block(start);
        level.mark_buddy_as_used(start);

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
