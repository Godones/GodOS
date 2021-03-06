#![no_std]
#![no_main]

use lib::{close, fstat, open, println, OpenFlags, Stat, StatMode};

/// 测试 fstat，输出　Test fstat OK! 就算正确。

#[no_mangle]
pub fn main() -> i32 {
    let fname = "fname1\0";
    let fd = open(fname, OpenFlags::C | OpenFlags::W);
    assert!(fd > 0);
    let fd = fd as usize;
    let stat: Stat = Stat::new();
    let ret = fstat(fd, &stat);
    assert_eq!(ret, 0);
    assert_eq!(stat.mode, StatMode::FILE);
    close(fd);
    assert_eq!(stat.nlink, 1);
    // unlink(fname);
    // It's recommended to rebuild the disk image. This program will not clean the file "fname1".
    println!("Test fstat OK!");
    0
}
