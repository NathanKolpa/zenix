use core::arch::asm;

/// Call the `hlt` instruction.
/// Note: the CPU will continue executing after handling an interrupt.
pub fn halt() {
    unsafe { asm!("hlt") }
}

/// Run the `hlt` instruction in a loop, ensuring the function will never exit.
pub fn halt_loop() -> ! {
    loop {
        halt()
    }
}
