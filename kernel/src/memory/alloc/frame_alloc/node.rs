use crate::util::address::PhysicalAddress;

pub struct FreeListNode {
    pub next: Option<PhysicalAddress>,
}
