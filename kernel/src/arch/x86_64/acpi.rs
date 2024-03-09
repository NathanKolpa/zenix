mod acpi_error;
mod acpi_info;

pub use acpi_error::AcpiError;
pub use acpi_info::AcpiInfo;

use bootinfo::BootInfo;
use essentials::{address::VirtualAddress, PanicOnce};
use x86_64::acpi::RSDP;

pub static ACPI_INFO: PanicOnce<Option<AcpiInfo>> = PanicOnce::new();

unsafe fn parse_acpi(rsdp_addr: VirtualAddress) -> Result<AcpiInfo, AcpiError> {
    let rsdp = &*rsdp_addr.as_ptr::<RSDP>();

    if !rsdp.checksum_ok() {
        return Err(AcpiError::RsdpChecksum);
    }

    Ok(AcpiInfo {
        oem_id: rsdp.oem_id,
    })
}

pub unsafe fn init_acpi(bootinfo: &BootInfo) -> Result<(), AcpiError> {
    let Some(rsdp_addr) = bootinfo.rsdp_addr() else {
        ACPI_INFO.initialize_with(None);
        return Ok(());
    };

    let parse_result = parse_acpi(rsdp_addr);
    let err = parse_result.as_ref().err().copied();

    ACPI_INFO.initialize_with(parse_result.ok());

    if let Some(err) = err {
        return Err(err);
    }

    Ok(())
}
