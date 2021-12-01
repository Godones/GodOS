use crate::mm::page_table::UserBuffer;

mod pipe;
mod stdio;

pub use pipe::Pipe;
pub use stdio::{Stdin, Stdout};

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
