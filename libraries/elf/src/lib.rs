#![cfg_attr(not(test), no_std)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner::runner)]

mod reader;
mod structure;

pub use reader::header::*;
pub use reader::ident::*;
pub use reader::program_header::*;
pub use reader::ElfReadError;
