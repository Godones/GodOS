use crate::file::File;
use crate::mm::page_table::UserBuffer;
use crate::print;
use crate::sbi::console_getchar;
use crate::task::suspend_current_run_next;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            //写入用户地址空间的缓冲区中
            buf.buffer[0].as_mut_ptr().write_volatile(ch)
        }
        1
    }
    //write负责将buf中的内容写入文件中
    fn write(&self, _buf: UserBuffer) -> usize {
        panic!("Stdin unsupported write");
    }
}

impl File for Stdout {
    fn read(&self, _buf: UserBuffer) -> usize {
        panic!("Stdout unsupported read");
    }
    fn write(&self, buf: UserBuffer) -> usize {
        for val in buf.buffer.iter() {
            print!("{}", core::str::from_utf8(*val).unwrap());
        }
        buf.len()
    }
}
