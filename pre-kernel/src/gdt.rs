use x86_64::segmentation::*;

use crate::bump_memory::BumpMemory;

pub struct InitialGdt {
    pub table: &'static GlobalDescriptorTable,
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
}

pub fn setup_gdt_table(bump_memory: &mut BumpMemory) -> InitialGdt {
    let table_bytes = bump_memory.alloc_struct::<GlobalDescriptorTable>();

    let table = table_bytes.write(GlobalDescriptorTable::new());
    let kernel_code = table.add_entry(SegmentDescriptor::KERNEL_CODE).unwrap();
    let kernel_data = table.add_entry(SegmentDescriptor::KERNEL_DATA).unwrap();

    InitialGdt {
        table,
        kernel_code,
        kernel_data,
    }
}
