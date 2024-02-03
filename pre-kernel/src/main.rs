#![no_std]
#![no_main]

mod vga;

use core::arch::global_asm;

global_asm!(include_str!("boot.s"));

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
struct MultibootInfo {
    flags: u32,
    mem_lower: u32,
    mem_upper: u32,
    boot_device: u32,
    cmdline: u32,
}
#[no_mangle]
pub extern "fastcall" fn main(multiboot_magic_arg: u32, multiboot_info_addr: u32) {
    vga::clear_screen();
    vga::set_running_msg();

    if multiboot_magic_arg != 0x2BADB002 {
        vga::set_fail_msg("Multiboot magic value in EAX does not equal 0x2BADB002");
        return;
    }

    let info_ptr = multiboot_info_addr as *const MultibootInfo;
    let info = unsafe { &*info_ptr };
}
