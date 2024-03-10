#[macro_export]
macro_rules! wrap_isr {
    ($outer:ident, $def:ident) => {
        #[naked]
        pub extern "x86-interrupt" fn $def(_frame: InterruptStackFrame) {
            fn inner(ctx: *const x86_64::interrupt::InterruptedContext) -> *const x86_64::interrupt::InterruptedContext {
                let new_ctx = $outer(ctx);
                (&new_ctx) as *const _
            }

            unsafe {
                core::arch::asm!(
                    "push rax",
                    "push rbx",
                    "push rcx",
                    "push rdx",
                    "push rdi",
                    "push rsi",
                    "push rbp",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",
                    "push r12",
                    "push r13",
                    "push r14",
                    "push r15",
                    //
                    "mov rdi, rsp",
                    "call {handler}",
                    "cmp rax, 0",
                    "je 2f",
                    "mov rsp, rax",
                    "2:",
                    //
                    "pop r15",
                    "pop r14",
                    "pop r13",
                    "pop r12",
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rbp",
                    "pop rsi",
                    "pop rdi",
                    "pop rdx",
                    "pop rcx",
                    "pop rbx",
                    "pop rax",
                    "iretq",
                    handler = sym inner,
                    options(noreturn)
                );
            }
        }
    };
}
