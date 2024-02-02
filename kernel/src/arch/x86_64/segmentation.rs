use core::ptr::addr_of_mut;

pub use gdt::*;
pub use segment::*;
pub use selector::*;
pub use tss::*;

use crate::arch::x86_64::PrivilegeLevel;
use crate::util::spin::Singleton;

mod gdt;
mod segment;
mod selector;
mod tss;

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;
pub const PAGE_FAULT_IST_INDEX: usize = 1;

fn init_tss() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();

    static mut DOUBLE_FAULT_STACK: [u8; 4096 * 16] = [0; 4096 * 16];

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
        TssStackPointer::from_slice(unsafe { &mut *addr_of_mut!(DOUBLE_FAULT_STACK) });

    static mut PAGE_FAULT_STACK: [u8; 4096 * 16] = [0; 4096 * 16];

    tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX] =
        TssStackPointer::from_slice(unsafe { &mut *addr_of_mut!(PAGE_FAULT_STACK) });

    static mut PRIVILEGED_STACK: [u8; 4096 * 16] = [0; 4096 * 16]; // TODO: dynamically allocate this.

    tss.privilege_stack_table[0] =
        TssStackPointer::from_slice(unsafe { &mut *addr_of_mut!(PRIVILEGED_STACK) });

    tss
}

pub static TSS: Singleton<TaskStateSegment> = Singleton::new(init_tss);

pub struct FullGdt {
    pub table: GlobalDescriptorTable,
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss: SegmentSelector,
    pub syscall: SegmentSelector,
    pub sysret: SegmentSelector,
}

impl FullGdt {
    /// Loads the GDT and sets all segments (as kernel) to point to the GDT.
    pub fn load(&'static self) {
        self.table.load();

        // Safety: we know that the segments point to a valid GDT as we loaded it just above.
        // And since self is static, we know it will remain that way
        unsafe {
            GDT.kernel_code.load_into_cs();
            GDT.kernel_data.load_into_ss();
            GDT.kernel_data.load_into_ds();
            GDT.tss.load_into_tss();
        }
    }
}

fn init_gdt() -> FullGdt {
    let mut table = GlobalDescriptorTable::new();

    let kernel_code = table.add_entry(SegmentDescriptor::KERNEL_CODE).unwrap();
    // Kernel data is required by syscall to be the next entry after kernel code.
    let kernel_data = table.add_entry(SegmentDescriptor::KERNEL_DATA).unwrap();

    // User data is required by sysret to the next entry after the selector.
    let user_data = table.add_entry(SegmentDescriptor::USER_DATA).unwrap();
    // User code is required by sysret to be the next entry after user data.
    let user_code = table.add_entry(SegmentDescriptor::USER_CODE).unwrap();

    let tss = table.add_entry(SegmentDescriptor::new_tss(&TSS)).unwrap();

    FullGdt {
        table,
        kernel_code,
        kernel_data,
        user_code,
        user_data,
        tss,
        syscall: SegmentSelector::new(kernel_code.index(), PrivilegeLevel::Ring0),
        sysret: SegmentSelector::new(kernel_data.index(), PrivilegeLevel::Ring3),
    }
}

pub static GDT: Singleton<FullGdt> = Singleton::new(init_gdt);
