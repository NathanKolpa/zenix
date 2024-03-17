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

#[derive(Debug, PartialEq, Eq)]
pub struct Path {
    bytes: str,
}

impl Path {
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

        unsafe { Ok(&*(str as *const str as *const Self)) }
    }
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
