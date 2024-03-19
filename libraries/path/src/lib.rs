#![cfg_attr(not(test), no_std)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner::runner)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    InvalidChar(usize, char),
    TooManyDots(usize),
    NoSlash(usize),
    Empty,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path {
    bytes: str,
}

impl Path {
    pub const ROOT: &'static Path = unsafe { Path::new_unchecked("/") };

    pub fn new<S: AsRef<str> + ?Sized>(value: &S) -> Result<&Self, PathError> {
        let str = value.as_ref();

        let mut dot_count: u8 = 0;
        for (index, byte) in str.chars().enumerate() {
            match (dot_count, byte) {
                (_, '/') => {
                    dot_count = 0;
                }
                (2, '.') => return Err(PathError::TooManyDots(index)),
                (_, '.') => {
                    dot_count += 1;
                }
                (0, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-') => {}
                _ => {
                    if dot_count > 0 {
                        return Err(PathError::NoSlash(index));
                    }

                    return Err(PathError::InvalidChar(index, byte));
                }
            }
        }

        Ok(unsafe { Self::new_unchecked(str) })
    }

    const unsafe fn new_unchecked(str: &str) -> &Self {
        &*(str as *const str as *const Self)
    }

    pub fn is_absolute(&self) -> bool {
        !self.is_relative()
    }

    pub fn is_relative(&self) -> bool {
        self.bytes.as_bytes()[0] != b'/'
    }

    pub fn as_str(&self) -> &str {
        &self.bytes
    }

    pub fn starts_with(&self, base: &Path) -> bool {
        let mut components = self.components();

        for base_component in base.components() {
            let Some(component) = components.next() else {
                return false;
            };

            if base_component != component {
                return false;
            }
        }

        true
    }

    pub fn components(&self) -> Components<'_> {
        Components {
            path: self,
            index: 0,
        }
    }

    pub fn combine(&self, _other: &Path) -> (&Path, &Path) {
        todo!()
    }
}

#[allow(dead_code)]
pub struct Components<'a> {
    path: &'a Path,
    index: usize,
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component<'a> {
    RootDir,
    CurrentDir,
    Parent,
    Name(&'a str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_root_path() {
        assert!(Path::new("/").is_ok());
    }

    #[test_case]
    fn test_multiple_root_slahses() {
        assert!(Path::new("/////////").is_ok());
    }

    #[test_case]
    fn test_dot() {
        assert!(Path::new(".").is_ok());
    }

    #[test_case]
    fn test_dot_dot() {
        assert!(Path::new("..").is_ok());
    }
    #[test_case]
    fn test_dot_slash() {
        assert!(Path::new("./").is_ok());
    }

    #[test_case]
    fn test_dot_dot_slash() {
        assert!(Path::new("../").is_ok());
    }

    #[test_case]
    fn test_dot_dot_dot_slash() {
        assert_eq!(Err(PathError::TooManyDots(2)), Path::new(".../"));
    }

    #[test_case]
    fn test_dot_slash_dot_dot() {
        assert!(Path::new("./..").is_ok());
    }

    #[test_case]
    fn test_dot_slash_many() {
        assert!(Path::new("./././././././.").is_ok());
    }

    const VALID_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
    const SPECIAL_CHARS: &str = "/.";

    #[test_case]
    fn test_valid_characters() {
        assert!(Path::new(VALID_CHARS).is_ok());
    }

    #[test_case]
    fn test_invalid_character() {
        assert_eq!(Err(PathError::InvalidChar(0, '#')), Path::new("#"));
    }

    #[test_case]
    fn test_utf8_fuzzing() {
        let mut buff = [0u8; 64];

        for code_point in 0..=0x10FFFF {
            let Some(utf8_char) = char::from_u32(code_point) else {
                continue;
            };

            if VALID_CHARS.contains(utf8_char) || SPECIAL_CHARS.contains(utf8_char) {
                continue;
            }

            let char_str = utf8_char.encode_utf8(&mut buff);

            assert_eq!(
                Err(PathError::InvalidChar(0, utf8_char)),
                Path::new(char_str)
            );
        }
    }
}
