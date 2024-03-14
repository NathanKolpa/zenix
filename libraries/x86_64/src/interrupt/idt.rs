use crate::interrupt::{DivergingErrorIsr, ErrorIsr, GateDescriptor, Isr, PageFaultIsr};
use crate::DescriptorTablePointer;
use core::mem::size_of;
use core::ops::{Index, IndexMut};
use essentials::address::VirtualAddress;

#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_error: GateDescriptor<Isr>,
    pub debug: GateDescriptor<Isr>,
    pub non_maskable_interrupt: GateDescriptor<Isr>,
    pub breakpoint: GateDescriptor<Isr>,
    pub overflow: GateDescriptor<Isr>,
    pub bound_range_exceeded: GateDescriptor<Isr>,
    pub invalid_opcode: GateDescriptor<Isr>,
    pub device_not_available: GateDescriptor<Isr>,
    pub double_fault: GateDescriptor<DivergingErrorIsr>,
    pub coprocessor_segment_overrun: GateDescriptor<Isr>,
    pub invalid_tss: GateDescriptor<Isr>,
    pub segment_not_present: GateDescriptor<Isr>,
    pub stack_segment_fault: GateDescriptor<Isr>,
    pub general_protection_fault: GateDescriptor<ErrorIsr>,
    pub page_fault: GateDescriptor<PageFaultIsr>,
    pub reserved_1: GateDescriptor<Isr>,
    pub x87_floating_point: GateDescriptor<Isr>,
    pub alignment_check: GateDescriptor<Isr>,
    pub machine_check: GateDescriptor<Isr>,
    pub simd_floating_point: GateDescriptor<Isr>,
    pub virtualization: GateDescriptor<Isr>,
    pub reserved_2: [GateDescriptor<Isr>; 8],
    pub vmm_communication_exception: GateDescriptor<Isr>,
    pub security_exception: GateDescriptor<Isr>,
    pub reserved_3: GateDescriptor<Isr>,
    interrupts: [GateDescriptor<Isr>; Self::USER_INTERRUPTS_COUNT],
}

impl InterruptDescriptorTable {
    pub const STANDARD_INTERRUPTS_COUNT: usize = 32;
    const USER_INTERRUPTS_COUNT: usize = 256 - Self::STANDARD_INTERRUPTS_COUNT;

    pub const fn new() -> Self {
        Self {
            divide_error: GateDescriptor::new(),
            debug: GateDescriptor::new(),
            non_maskable_interrupt: GateDescriptor::new(),
            breakpoint: GateDescriptor::new(),
            overflow: GateDescriptor::new(),
            bound_range_exceeded: GateDescriptor::new(),
            invalid_opcode: GateDescriptor::new(),
            device_not_available: GateDescriptor::new(),
            double_fault: GateDescriptor::new(),
            coprocessor_segment_overrun: GateDescriptor::new(),
            invalid_tss: GateDescriptor::new(),
            segment_not_present: GateDescriptor::new(),
            stack_segment_fault: GateDescriptor::new(),
            general_protection_fault: GateDescriptor::new(),
            page_fault: GateDescriptor::new(),
            reserved_1: GateDescriptor::new(),
            x87_floating_point: GateDescriptor::new(),
            alignment_check: GateDescriptor::new(),
            machine_check: GateDescriptor::new(),
            simd_floating_point: GateDescriptor::new(),
            virtualization: GateDescriptor::new(),
            reserved_2: [GateDescriptor::new(); 8],
            vmm_communication_exception: GateDescriptor::new(),
            security_exception: GateDescriptor::new(),
            reserved_3: GateDescriptor::new(),
            interrupts: [GateDescriptor::new(); Self::USER_INTERRUPTS_COUNT],
        }
    }

    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer::new(
            (size_of::<Self>() - 1) as u16,
            VirtualAddress::from(self as *const _),
        )
    }

    pub fn load(&'static self) {
        unsafe { self.load_unsafe() }
    }

    /// # Safety
    ///
    /// The caller must ensure that no interrupts are generated after the table is destructed.
    pub unsafe fn load_unsafe(&self) {
        let pointer = self.pointer();
        pointer.load_interrupt_table();
    }

    pub fn isr_iter_mut(&mut self) -> core::slice::IterMut<'_, GateDescriptor<Isr>> {
        self.interrupts.iter_mut()
    }

    pub fn isr_iter(&self) -> core::slice::Iter<'_, GateDescriptor<Isr>> {
        self.interrupts.iter()
    }
}

impl Index<usize> for InterruptDescriptorTable {
    type Output = GateDescriptor<Isr>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.interrupts[index - Self::STANDARD_INTERRUPTS_COUNT]
    }
}

impl IndexMut<usize> for InterruptDescriptorTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.interrupts[index - Self::STANDARD_INTERRUPTS_COUNT]
    }
}
