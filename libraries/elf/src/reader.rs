mod error;
pub mod header;
pub mod ident;
pub mod program_header;
pub mod section_header;

pub use error::ElfReadError;

/// # Safety
///
/// The caller must ensure that type `T` only contains primitives like `u32` or `u8`.
/// If type `T` contains values like an enum, undefined behaviour can occour.
unsafe fn read_struct<T: Clone>(raw_data: &[u8], offset: usize) -> Result<T, ElfReadError> {
    use core::mem::{align_of, size_of};

    let value_data = &raw_data[offset..];

    if value_data.len() < size_of::<T>() {
        return Err(ElfReadError::TooSmall);
    }

    if value_data.as_ptr() as usize % align_of::<T>() != 0 {
        return Err(ElfReadError::NotAligned);
    }

    Ok((*(value_data.as_ptr() as *const T)).clone())
}
