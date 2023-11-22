use crate::arch::x86_64::interrupt::InterruptStackFrame;
use crate::arch::x86_64::segmentation::SegmentSelector;
use crate::util::address::VirtualAddress;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageFaultErrorCode {
    value: u64,
}

impl PageFaultErrorCode {
    pub fn protection_violation(&self) -> bool {
        self.value & 1 != 0
    }

    pub fn caused_by_write(&self) -> bool {
        self.value & (1 << 1) != 0
    }

    pub fn user_mode(&self) -> bool {
        self.value & (1 << 2) != 0
    }

    pub fn malformed_table(&self) -> bool {
        self.value & (1 << 3) != 0
    }

    pub fn instruction_fetch(&self) -> bool {
        self.value & (1 << 4) != 0
    }

    pub fn protection_key(&self) -> bool {
        self.value & (1 << 5) != 0
    }

    pub fn shadow_stack(&self) -> bool {
        self.value & (1 << 6) != 0
    }

    pub fn sgx_access_control(&self) -> bool {
        self.value & (1 << 7) != 0
    }

    pub fn rmp_violation(&self) -> bool {
        self.value & (1 << 31) != 0
    }

    pub fn as_u64(&self) -> u64 {
        self.value
    }
}

impl Debug for PageFaultErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("PageFaultErrorCode");
        ds.field("protection_violation", &self.protection_violation());
        ds.field("caused_by_write", &self.caused_by_write());
        ds.field("user_mode", &self.user_mode());
        ds.field("malformed_table", &self.malformed_table());
        ds.field("instruction_fetch", &self.instruction_fetch());
        ds.field("protection_key", &self.protection_key());
        ds.field("shadow_stack", &self.shadow_stack());
        ds.field("sgx_access_control", &self.sgx_access_control());
        ds.field("rmp_violation", &self.rmp_violation());
        ds.finish()
    }
}

pub type Isr = extern "x86-interrupt" fn(InterruptStackFrame);
pub type ErrorIsr = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
pub type PageFaultIsr =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErrorCode);
pub type DivergingErrorIsr = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64) -> !;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GateAttributes {
    type_attributes: u8,
}

impl GateAttributes {
    const INTERRUPT_GATE_TYPE: u8 = 0b1110;

    pub const fn new() -> Self {
        Self {
            type_attributes: Self::transform_type_bits(0, Self::INTERRUPT_GATE_TYPE),
        }
    }

    const fn transform_type_bits(original: u8, value: u8) -> u8 {
        original ^ ((original ^ value) & 0x0F)
    }

    fn set_type_bits(&mut self, value: u8) {
        self.type_attributes = Self::transform_type_bits(self.type_attributes, value)
    }

    fn make_interrupt_gate(&mut self) {
        self.set_type_bits(Self::INTERRUPT_GATE_TYPE);
    }

    fn make_trap_gate(&mut self) {
        self.set_type_bits(0b1111);
    }

    fn enable(&mut self) {
        self.type_attributes |= 1 << 7;
    }

    fn disable(&mut self) {
        self.type_attributes &= !(1 << 7);
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GateDescriptor<F> {
    offset_1: u16,
    selector: SegmentSelector,
    stack_table_offset: u8,
    attributes: GateAttributes,
    offset_2: u16,
    offset_3: u32,
    _reserved: u32,
    _phantom: PhantomData<F>,
}

impl<F> GateDescriptor<F> {
    pub const fn new() -> Self {
        Self {
            offset_1: 0,
            selector: SegmentSelector::empty(),
            stack_table_offset: 0,
            attributes: GateAttributes::new(),
            offset_2: 0,
            offset_3: 0,
            _reserved: 0,
            _phantom: PhantomData,
        }
    }

    fn set_handler_address(&mut self, addr: VirtualAddress) {
        let addr = addr.as_u64();

        self.offset_1 = addr as u16;
        self.offset_2 = (addr >> 16) as u16;
        self.offset_3 = (addr >> 32) as u32;
    }

    fn enable(&mut self, selector: SegmentSelector) {
        self.selector = selector;
        self.attributes.enable();
    }

    fn enable_as_normal(&mut self, selector: SegmentSelector) {
        self.enable(selector);
        self.attributes.make_interrupt_gate();
    }

    fn enable_as_trap(&mut self, selector: SegmentSelector) {
        self.enable(selector);
        self.attributes.make_trap_gate();
    }

    pub fn disable(&mut self) {
        self.attributes.disable();
    }

    pub fn set_stack_index(&mut self, index: usize) {
        self.stack_table_offset = index as u8 + 1;
    }
}

impl GateDescriptor<Isr> {
    pub fn set_handler(&mut self, selector: SegmentSelector, handler: Isr) {
        self.set_handler_address(VirtualAddress::from(handler as usize));
        self.enable_as_normal(selector);
    }
}

impl GateDescriptor<ErrorIsr> {
    pub fn set_handler(&mut self, selector: SegmentSelector, handler: ErrorIsr) {
        self.set_handler_address(VirtualAddress::from(handler as usize));
        self.enable_as_trap(selector);
    }
}

impl GateDescriptor<PageFaultIsr> {
    pub fn set_handler(&mut self, selector: SegmentSelector, handler: PageFaultIsr) {
        self.set_handler_address(VirtualAddress::from(handler as usize));
        self.enable_as_trap(selector);
    }
}

impl GateDescriptor<DivergingErrorIsr> {
    pub fn set_handler(&mut self, selector: SegmentSelector, handler: DivergingErrorIsr) {
        self.set_handler_address(VirtualAddress::from(handler as usize));
        self.enable_as_trap(selector);
    }
}
