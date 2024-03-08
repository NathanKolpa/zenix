use bootinfo::BootInfo;

use essentials::{address::VirtualAddress, spin::SpinLock, PanicOnce};
use x86_64::{
    acpi::rsdt::RSDP,
    cpuid::read_features,
    device::{apic::Apic, pic_8259::ChainedPic8259},
};

use crate::{info_println, warning_println};

pub enum InterruptControl {
    Pic(SpinLock<ChainedPic8259>),
    Apic(Apic),
}

pub static INTERRUPT_CONTROL: PanicOnce<InterruptControl> = PanicOnce::new();

#[derive(Debug)]
enum ApicInitError {
    RsdpChecksum,
}

unsafe fn parse_rspd(rsdp_addr: VirtualAddress) -> Result<(), ApicInitError> {
    let header = &*rsdp_addr.as_ptr::<RSDP>();

    if !header.checksum_ok() {
        return Err(ApicInitError::RsdpChecksum);
    }

    if let Ok(oem) = core::str::from_utf8(&header.oem_id) {
        info_println!("ACPI OEM: {oem}");
    }

    Ok(())
}

unsafe fn init_apic(
    rsdp_addr: VirtualAddress,
    pic: &mut ChainedPic8259,
) -> Result<Apic, ApicInitError> {
    pic.disable();

    parse_rspd(rsdp_addr)?;

    let mut apic = Apic::from_msr();
    apic.enable();

    Ok(apic)
}

pub unsafe fn init_interrupt_control(bootinfo: &BootInfo) {
    let cpu_features = read_features();

    let mut pic = ChainedPic8259::new(super::idt::IRQ_START as u8);
    pic.init();

    if let Some(rsdp) = bootinfo.rsdp_addr() {
        if cpu_features.apic() {
            match init_apic(rsdp, &mut pic) {
                Ok(apic) => {
                    INTERRUPT_CONTROL.initialize_with(InterruptControl::Apic(apic));
                    return;
                }
                Err(apic_err) => {
                    warning_println!("Failed to initialize APIC: {apic_err:?}");
                    pic.enable();
                }
            }
        }
    }

    INTERRUPT_CONTROL.initialize_with(InterruptControl::Pic(SpinLock::new(pic)));
}
