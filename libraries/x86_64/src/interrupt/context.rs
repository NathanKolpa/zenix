use crate::segmentation::SegmentSelector;
use crate::RFlags;
use essentials::address::VirtualAddress;

/// All cpu registers that are relevant when interrupting a program.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct RegisterContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
}

/// All data that gets pushed on the stack in an interrupt handler.
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtualAddress,
    pub code_segment: u64,
    pub cpu_flags: RFlags,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: u64,
}

impl InterruptStackFrame {
    pub fn new(
        instruction_pointer: VirtualAddress,
        stack_pointer: VirtualAddress,
        cpu_flags: RFlags,
        code_segment: SegmentSelector,
        stack_segment: SegmentSelector,
    ) -> Self {
        Self {
            instruction_pointer,
            code_segment: code_segment.as_u16() as u64,
            cpu_flags,
            stack_pointer,
            stack_segment: stack_segment.as_u16() as u64,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct InterruptedContext {
    pub registers: RegisterContext,
    pub interrupt_stack_frame: InterruptStackFrame,
}

impl InterruptedContext {
    pub fn start_new(interrupt_stack_frame: InterruptStackFrame) -> Self {
        Self {
            registers: RegisterContext {
                r15: 0,
                r14: 0,
                r13: 0,
                r12: 0,
                r11: 0,
                r10: 0,
                r9: 0,
                r8: 0,
                rbp: interrupt_stack_frame.stack_pointer.as_u64(),
                rsi: 0,
                rdi: 0,
                rdx: 0,
                rcx: 0,
                rbx: 0,
                rax: 0,
            },
            interrupt_stack_frame,
        }
    }
}
