use crate::mm::page_table::UserBuffer;

mod pipe;
mod stdio;
mod mail;
mod inode;
mod ftable;

pub use pipe::Pipe;
pub use stdio::{Stdin, Stdout};
pub use mail::Mail;
pub use inode::{
    list_apps,
    open_file,
    create_nlink_file,
    delete_nlink_file,
    OpenFlags,
    FNode
};
pub use ftable::*;

pub trait File: Send + Sync {
    fn read(&self,  buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn fstat(&self)->Stat;
}
