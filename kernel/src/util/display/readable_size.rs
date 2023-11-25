use core::cmp::min;
use core::fmt::{Debug, Display, Formatter};

pub struct ReadableSize {
    bytes: usize,
}

impl ReadableSize {
    const SIZES: &'static [&'static str] = &["B", "KiB", "MiB", "GiB", "TiB"];

    pub const fn new(bytes: usize) -> Self {
        Self { bytes }
    }

    fn magnitude(&self) -> usize {
        let scale: usize = 1024;

        if self.bytes == 0 {
            0
        } else {
            min(
                Self::SIZES.len() - 1,
                (self.bytes.ilog10() / scale.ilog10()) as usize,
            )
        }
    }

    pub fn unit(&self) -> &'static str {
        Self::SIZES[self.magnitude()]
    }
}

impl Display for ReadableSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut value: f64 = self.bytes as f64;

        for _ in 0..self.magnitude() {
            value /= 1024_f64;
        }

        write!(f, "{:.1} {}", value, self.unit())
    }
}

impl Debug for ReadableSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_zero_bytes_unit() {
        let size = ReadableSize::new(0);
        let unit = size.unit();
        assert_eq!("B", unit)
    }

    #[test_case]
    fn test_below_1_kib_unit() {
        let size = ReadableSize::new(500);
        let unit = size.unit();
        assert_eq!("B", unit)
    }

    #[test_case]
    fn test_exactly_1_kib_unit() {
        let size = ReadableSize::new(1024);
        let unit = size.unit();
        assert_eq!("KiB", unit)
    }

    #[test_case]
    fn test_2_kib_unit() {
        let size = ReadableSize::new(2048);
        let unit = size.unit();
        assert_eq!("KiB", unit)
    }

    #[test_case]
    fn test_exactly_1_mib_unit() {
        let size = ReadableSize::new(1024 * 1024);
        let unit = size.unit();
        assert_eq!("MiB", unit)
    }
}
