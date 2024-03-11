#[macro_export]
macro_rules! wrap_isr {
    ($outer:ident, $def:ident) => {
        #[naked]
        pub extern "x86-interrupt" fn $def(_frame: ::x86_64::interrupt::InterruptStackFrame) {
            use ::x86_64::interrupt::InterruptedContext;

            // wrapping the outer with lifetime '_ prevents unsound 'static lifetime
            fn forward(ctx: &InterruptedContext) -> Option<InterruptedContext> {
                $outer(ctx)
            }

            unsafe extern "C" fn inner(ctx: *mut InterruptedContext) {
                match forward(&*ctx) {
                    None => {},
                    Some(new_ctx) =>  {
                        *ctx = new_ctx;
                    },
                }
            }

            unsafe {
                core::arch::asm!(
                    // save
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
                    // ctx ptr
                    "mov rdi, rsp",
                    // call to handler
                    "call {handler}",
                    // restore
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
