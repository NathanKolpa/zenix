mod error;
mod parse;

pub use error::AcpiError;
pub use parse::AcpiInfo;

use bootinfo::BootInfo;
use essentials::PanicOnce;

use crate::memory::map::MemoryMapper;

pub static ACPI_INFO: PanicOnce<Option<AcpiInfo>> = PanicOnce::new();

pub unsafe fn init_acpi(bootinfo: &BootInfo, mapper: &mut MemoryMapper) -> Result<(), AcpiError> {
    let Some(rsdp_addr) = bootinfo.rsdp_addr() else {
        ACPI_INFO.initialize_with(None);
        return Ok(());
    };

    let parse_result = parse::parse_acpi(rsdp_addr, mapper);
    let err = parse_result.as_ref().err().copied();

    ACPI_INFO.initialize_with(parse_result.ok());

    if let Some(err) = err {
        return Err(err);
    }

    Ok(())
}
