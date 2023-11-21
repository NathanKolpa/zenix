use crate::arch::x86_64::segmentation::GDT;

/// Initialize x86_64 specific stuff.
pub fn init() {
    GDT.load();
}
