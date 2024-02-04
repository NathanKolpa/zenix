pub struct Bitmap<S> {
    bytes: S,
}

impl<S> Bitmap<S>
where
    S: AsMut<[u8]>,
{
    pub const fn new(storage: S) -> Self {
        Self { bytes: storage }
    }

    pub fn set(&mut self, bit: usize) {
        let byte_index = bit / 8;
        let bit_index = bit % 8;
        self.bytes.as_mut()[byte_index] |= 1 << bit_index;
    }

    pub fn clear(&mut self, bit: usize) {
        let byte_index = bit / 8;
        let bit_index = bit % 8;
        self.bytes.as_mut()[byte_index] &= !(1 << bit_index);
    }
}

impl<S> Bitmap<S>
where
    S: AsRef<[u8]>,
{
    pub fn contains(&self, bit: usize) -> bool {
        let byte_index = bit / 8;
        let bit_index = bit % 8;
        self.bytes.as_ref()[byte_index] & (1 << bit_index) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_initialize_to_zero() {
        let bitmap = Bitmap::new([0u8; 2]);
        assert!(!bitmap.contains(9));
    }

    #[test_case]
    fn test_set_and_check() {
        let mut bitmap = Bitmap::new([0u8; 2]);
        bitmap.set(9);
        assert!(bitmap.contains(9));

        // check surrounding bits
        assert!(!bitmap.contains(8));
        assert!(!bitmap.contains(10));
    }

    #[test_case]
    fn test_set_and_clear() {
        let mut bitmap = Bitmap::new([0u8; 2]);

        bitmap.set(9);
        bitmap.clear(9);

        // all should be empty
        assert!(!bitmap.contains(9));
        assert!(!bitmap.contains(8));
        assert!(!bitmap.contains(10));
    }
}
