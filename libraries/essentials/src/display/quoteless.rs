use core::fmt::{Debug, Display};

pub struct Quoteless<'a> {
    value: &'a str,
}

impl<'a> Quoteless<'a> {
    pub fn new(value: &'a str) -> Self {
        Self { value }
    }
}

impl Display for Quoteless<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Debug for Quoteless<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}
