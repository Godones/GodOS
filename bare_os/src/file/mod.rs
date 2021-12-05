use crate::mm::page_table::UserBuffer;

mod pipe;
mod stdio;
mod mail;

pub use pipe::Pipe;
pub use stdio::{Stdin, Stdout};
pub use mail::Mail;
pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
