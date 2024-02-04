use core::mem::size_of;
use essentials::address::VirtualAddress;

/// A pointer to a stack that can be used by the [`TaskStateSegment`].
///
/// From the [osdev wiki](https://wiki.osdev.org/Task_State_Segment):
/// >  The Stack Pointers used to load the stack when an entry in the Interrupt Descriptor Table has an IST value other than 0.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TssStackPointer {
    addr: VirtualAddress,
}

impl TssStackPointer {
    pub const fn null() -> Self {
        Self {
            addr: VirtualAddress::new(0),
        }
    }

    pub fn from_slice(stack: &'static mut [u8]) -> Self {
        let start = VirtualAddress::from(stack.as_ptr());
        let end = start + stack.len();

        Self { addr: end }
    }

    pub fn stack_end(&self) -> VirtualAddress {
        self.addr
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    _reserved_1: u32,
    pub privilege_stack_table: [TssStackPointer; 3],
    _reserved_2: u64,
    pub interrupt_stack_table: [TssStackPointer; 7],
    _reserved_3: u64,
    _reserved_4: u16,
    io_map_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        TaskStateSegment {
            _reserved_1: 0,
            privilege_stack_table: [TssStackPointer::null(); 3],
            _reserved_2: 0,
            interrupt_stack_table: [TssStackPointer::null(); 7],
            _reserved_3: 0,
            _reserved_4: 0,
            io_map_base: size_of::<Self>() as u16,
        }
    }
}
