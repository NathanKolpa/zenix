//! Abstractions around `void` pointers.

use core::ops::*;
use core::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};

pub trait AddressMarker {
    const SIZE: usize;

    /// From phill opp's blog:
    /// > Even though bits 48 to 64 are discarded, they can’t be set to arbitrary values.
    /// > Instead, all bits in this range have to be copies of bit 47 in order to keep addresses unique and allow future extensions like the 5-level page table.
    /// > This is called sign-extension because it’s very similar to the sign extension in two’s complement.
    /// > When an address is not correctly sign-extended, the CPU throws an exception.
    const SING_EXT: bool;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddressMarker {}

impl AddressMarker for VirtualAddressMarker {
    #[cfg(target_arch = "x86_64")]
    const SIZE: usize = 48;

    #[cfg(target_arch = "x86_64")]
    const SING_EXT: bool = true;

    #[cfg(target_arch = "x86")]
    const SIZE: usize = 32;

    #[cfg(target_arch = "x86")]
    const SING_EXT: bool = false;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddressMarker {}

impl AddressMarker for PhysicalAddressMarker {
    #[cfg(target_arch = "x86_64")]
    const SIZE: usize = 52;
    #[cfg(target_arch = "x86_64")]
    const SING_EXT: bool = false;

    #[cfg(target_arch = "x86")]
    const SIZE: usize = 32;
    #[cfg(target_arch = "x86")]
    const SING_EXT: bool = false;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub struct Address<L> {
    addr: usize,
    _phantom: PhantomData<L>,
}

pub type VirtualAddress = Address<VirtualAddressMarker>;
pub type PhysicalAddress = Address<PhysicalAddressMarker>;

/// A generic memory address.
///
/// Physical and Virtual addresses are split up using the generic argument `L` for the following reasons:
/// - You can dereference only virtual addresses and not physical.
/// - Their internal representation differs.
///
///  See [paging introduction](https://os.phil-opp.com/paging-introduction/#paging-on-x86-64) for a more detailed explanation.
impl<L> Address<L>
where
    L: AddressMarker,
{
    const ADDR_MASK: usize = usize::MAX >> (usize::BITS as usize - L::SIZE);
    const VALUE_MASK: usize = !Self::ADDR_MASK;

    pub const fn new(addr: usize) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    fn new_modified(mut addr: usize) -> Self {
        addr &= Self::ADDR_MASK;

        if L::SING_EXT {
            let multiplier = Self::ADDR_MASK & addr >> L::SIZE.saturating_sub(1);
            addr |= multiplier * Self::VALUE_MASK;
        }

        Self::new(addr)
    }

    pub const fn null() -> Self {
        Self {
            addr: 0,
            _phantom: PhantomData,
        }
    }

    pub const fn max() -> Self {
        Self {
            addr: Self::ADDR_MASK,
            _phantom: PhantomData,
        }
    }

    pub const fn is_within(&self, region_addr: Address<L>, region_len: usize) -> bool {
        self.addr >= region_addr.addr && self.addr <= region_addr.addr + region_len
    }

    /// Aligns a memory address upwards to the specified alignment.
    ///
    /// # Parameters
    ///
    /// - `addr`: The memory address to be aligned.
    /// - `align`: The alignment value. It must be a power of two, otherwise, the function will panic.
    pub fn align_ptr_up(addr: usize, align: usize) -> usize {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        (addr + align - 1) & !(align - 1)
    }

    /// Aligns a memory address downwards to the specified alignment.
    ///
    /// # Parameters
    ///
    /// - `addr`: The memory address to be aligned.
    /// - `align`: The alignment value. It must be a power of two, otherwise, the function will panic.
    pub const fn align_ptr_down(addr: usize, align: usize) -> usize {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        addr & !(align - 1)
    }

    pub const fn as_u64(&self) -> u64 {
        self.addr as u64
    }

    pub fn as_usize(&self) -> usize {
        self.addr
    }

    pub fn is_aligned_with(&self, value: usize) -> bool {
        self.addr % value == 0
    }

    pub fn is_null(&self) -> bool {
        self.addr == 0
    }

    pub fn align_down(&self, align: usize) -> Self {
        Self::new_modified(Self::align_ptr_down(self.addr, align))
    }

    pub fn align_up(&self, align: usize) -> Self {
        Self::new_modified(Self::align_ptr_up(self.addr, align))
    }

    pub fn checked_add(&self, rhs: usize) -> Option<Self> {
        let value = self.addr.checked_add(rhs)?;

        if ((rhs & Self::ADDR_MASK) + (self.addr & Self::ADDR_MASK)) & Self::ADDR_MASK
            > usize::MAX & Self::ADDR_MASK
        {
            return None;
        }

        Some(Self::new_modified(value))
    }
}

impl<L: AddressMarker> Default for Address<L> {
    fn default() -> Self {
        Self::null()
    }
}

/// A wrapper for physical addresses.
impl Address<PhysicalAddressMarker> {}

/// A wrapper for virtual addresses, or normal pointers.
impl Address<VirtualAddressMarker> {
    #[cfg(target_arch = "x86_64")]
    #[doc(cfg(target_arch = "x86_64"))]
    fn truncate_index(value: usize) -> usize {
        value % 512
    }

    /// Get the x
    ///
    /// The page table indices are used to navigate through the hierarchy of page tables in the x86_64 paging structure.
    ///
    /// # Returns
    ///
    /// An array of type `[u16; 4]` containing the x86_64 page table indices. The indices in the array are ordered from highest
    /// level (page table 4) to lowest level (page table 1).
    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn indices(&self) -> [u16; 4] {
        [
            Self::truncate_index(self.addr >> 12 >> 9 >> 9 >> 9) as u16,
            Self::truncate_index(self.addr >> 12 >> 9 >> 9) as u16,
            Self::truncate_index(self.addr >> 12 >> 9) as u16,
            Self::truncate_index(self.addr >> 12) as u16,
        ]
    }

    /// Calculate the page offset for a virtual memory address on the x86_64 architecture.
    ///
    /// This function calculates and returns the page offset for a given virtual memory address
    /// on the x86_64 architecture. Assuming the page is not "huge".
    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn page_offset(&self) -> usize {
        (self.addr) % (1 << 12)
    }

    /// Calculate the page offset assuming the level 3 page table is "huge"
    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn l3_page_offset(&self) -> usize {
        self.addr & 0o_777_777_7777
    }

    /// Calculate the page offset assuming the level 2 page table is "huge"
    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn l2_page_offset(&self) -> usize {
        self.addr & 0o_777_7777
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.as_u64() as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.as_u64() as *mut T
    }

    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn from_l4_index(index: u16) -> Self {
        Self::new((index as usize) << (12 + 9 + 9 + 9))
    }

    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn from_indices(indices: [u16; 4]) -> Self {
        Self::new(
            (indices[0] as usize) << (12 + 9 + 9 + 9)
                | (indices[1] as usize) << (12 + 9 + 9)
                | (indices[2] as usize) << (12 + 9)
                | (indices[3] as usize) << (12),
        )
    }
}

impl<T: AddressMarker> Add<usize> for Address<T> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new_modified(self.addr + rhs)
    }
}

impl<T: AddressMarker> AddAssign<usize> for Address<T> {
    fn add_assign(&mut self, rhs: usize) {
        self.addr += rhs;
    }
}

impl<T: AddressMarker> Add<Address<T>> for Address<T> {
    type Output = Self;

    fn add(self, rhs: Address<T>) -> Self::Output {
        Self::new_modified(self.addr + rhs.addr)
    }
}

impl<T: AddressMarker> Sub<usize> for Address<T> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self::new_modified(self.addr - rhs)
    }
}

impl<T: AddressMarker> Sub<Address<T>> for Address<T> {
    type Output = Self;

    fn sub(self, rhs: Address<T>) -> Self::Output {
        Self::new_modified(self.addr - rhs.addr)
    }
}

impl<L: AddressMarker> From<usize> for Address<L> {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl<L: AddressMarker> From<u64> for Address<L> {
    fn from(value: u64) -> Self {
        Self::new(value as usize)
    }
}

impl<L: AddressMarker, T> From<*const T> for Address<L> {
    fn from(value: *const T) -> Self {
        Self::new(value as usize)
    }
}

impl<L: AddressMarker, T> From<*mut T> for Address<L> {
    fn from(value: *mut T) -> Self {
        Self::new(value as usize)
    }
}

impl<L> From<Address<L>> for usize {
    fn from(val: Address<L>) -> Self {
        val.addr
    }
}

impl<T> Display for Address<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.addr)
    }
}

impl Debug for Address<PhysicalAddressMarker> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "p{}", self)
    }
}

impl Debug for Address<VirtualAddressMarker> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test_case]
    fn test_align_zero_down() {
        assert_eq!(0, VirtualAddress::align_ptr_down(0, 1024));
    }

    #[test_case]
    fn test_align_one_down() {
        assert_eq!(0, VirtualAddress::align_ptr_down(1, 1024));
    }

    #[test_case]
    fn test_align_second_page_down() {
        assert_eq!(1024, VirtualAddress::align_ptr_down(1500, 1024));
    }

    #[test_case]
    fn test_align_zero_up() {
        assert_eq!(0, VirtualAddress::align_ptr_up(0, 1024));
    }

    #[test_case]
    fn test_align_one_up() {
        assert_eq!(1024, VirtualAddress::align_ptr_up(1, 1024));
    }

    #[test_case]
    fn test_from_index_and_indices() {
        let addr = VirtualAddress::from_l4_index(16);
        let indices = addr.indices();
        assert_eq!(16, indices[0]);
    }

    #[test_case]
    fn test_from_indices() {
        let addr = VirtualAddress::new(0xdeadbeef);

        let source_indices = addr.indices();
        let new_addr = VirtualAddress::from_indices(source_indices);
        let new_indices = new_addr.indices();
        assert_eq!(source_indices, new_indices);
    }

    #[test_case]
    #[cfg(target_arch = "x86_64")]
    fn test_sing_ext() {
        let last_bit_set = 1usize << 47;
        let addr = VirtualAddress::new(last_bit_set);

        assert_eq!(
            (addr.as_u64() & VirtualAddress::VALUE_MASK as u64)
                >> VirtualAddressMarker::SIZE as u64,
            VirtualAddress::VALUE_MASK as u64 >> VirtualAddressMarker::SIZE as u64
        )
    }
}
