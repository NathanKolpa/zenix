mod rsdp;

mod sdt_header;
mod sdt_madt;
mod sdt_root;

pub use rsdp::*;
pub use sdt_header::*;
pub use sdt_madt::*;
pub use sdt_root::*;

fn sum_bytes(raw_bytes: impl Iterator<Item = u8>) -> u8 {
    raw_bytes.fold(0u8, |acc, byte| acc.wrapping_add(byte))
}
