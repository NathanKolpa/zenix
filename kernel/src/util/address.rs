//! Abstractions around pointers.

use core::ops::Sub;
use core::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ops::Add,
};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddressMarker {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddressMarker {}

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
impl<L> Address<L> {
    pub const fn new(addr: usize) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }

    pub const fn null() -> Self {
        Self {
            addr: 0,
            _phantom: PhantomData,
        }
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
    pub fn align_ptr_down(addr: usize, align: usize) -> usize {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        addr & !(align - 1)
    }

    #[cfg(target_arch = "x86_64")]
    #[doc(cfg(target_arch = "x86_64"))]
    pub fn as_u64(&self) -> u64 {
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
}

/// A wrapper for physical addresses.
impl Address<PhysicalAddressMarker> {
    pub fn align_down(&self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");
        Self::new(Self::align_ptr_down(self.addr, align))
    }
}

/// A wrapper for virtual addresses, or normal pointers.
impl Address<VirtualAddressMarker> {
    pub fn align_down(&self, align: usize) -> Self {
        assert!(align.is_power_of_two(), "`align` must be a power of two");

        let mut addr = self.addr;
        addr = Self::align_ptr_down(addr, align);
        addr = ((addr << 16) as isize >> 16) as usize;

        Self::new(addr)
    }

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

    pub fn as_ptr<T>(&self) -> *const T {
        self.as_u64() as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.as_u64() as *mut T
    }

    #[doc(cfg(target_arch = "x86_64"))]
    #[cfg(target_arch = "x86_64")]
    pub fn from_l4_index(index: u16) -> Self {
        Self::new((index as usize) << (12 + 9 + 9 + 9))
    }
}

impl<T> Add<usize> for Address<T> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new(self.addr + rhs)
    }
}

impl<T> Add<Address<T>> for Address<T> {
    type Output = Self;

    fn add(self, rhs: Address<T>) -> Self::Output {
        Self::new(self.addr + rhs.addr)
    }
}

impl<T> Sub<usize> for Address<T> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self::new(self.addr - rhs)
    }
}

impl<T> Sub<Address<T>> for Address<T> {
    type Output = Self;

    fn sub(self, rhs: Address<T>) -> Self::Output {
        Self::new(self.addr - rhs.addr)
    }
}

impl<L> From<usize> for Address<L> {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl<L> From<u64> for Address<L> {
    fn from(value: u64) -> Self {
        Self::new(value as usize)
    }
}

impl<L, T> From<*const T> for Address<L> {
    fn from(value: *const T) -> Self {
        Self::new(value as usize)
    }
}

impl<L, T> From<*mut T> for Address<L> {
    fn from(value: *mut T) -> Self {
        Self::new(value as usize)
    }
}

impl<L> Into<usize> for Address<L> {
    fn into(self) -> usize {
        self.addr
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
}
