use x86_64::{cpuid, rdmsr};

use crate::info_println;

pub fn print_info() {
    let features = cpuid::read_features();
    info_println!("CPUID features: {features}");

    if features.apic() {
        info_println!("APIC: {}", rdmsr::read_apic_base());
    }
}
