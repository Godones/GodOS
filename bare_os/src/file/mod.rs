use crate::mm::page_table::UserBuffer;

mod ftable;
mod inode;
mod mail;
mod pipe;
mod stdio;

pub use ftable::*;

pub use inode::{create_nlink_file, delete_nlink_file, list_apps, open_file, FNode, OpenFlags};
pub use mail::Mail;
pub use pipe::Pipe;
pub use stdio::{Stdin, Stdout};

pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn fstat(&self) -> Stat;
}
