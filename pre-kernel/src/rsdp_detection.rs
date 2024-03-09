use x86_64::acpi::RSDP_SIGNATURE;

pub const RSDP_ADDR_START: usize = 0x000E0000;
pub const RSDP_ADDR_END: usize = 0x000FFFFF;

pub unsafe fn scan_special_region_for_rsdp() -> Option<u64> {
    let address_range = (RSDP_ADDR_START..RSDP_ADDR_END).step_by(16);

    for addr in address_range {
        let possible_signature = &*(addr as *const [u8; 8]);

        if possible_signature == &RSDP_SIGNATURE {
            return Some(addr as u64);
        }
    }

    None
}
