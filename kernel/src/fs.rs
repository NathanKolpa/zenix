pub mod fat;

mod vfs;

pub use vfs::*;

pub trait FileSystem {}
