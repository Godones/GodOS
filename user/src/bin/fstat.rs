#![no_std]
#![no_main]


use lib::{close, fstat, INFO, open, OpenFlags, println, Stat, StatMode};

/// 测试 fstat，输出　Test fstat OK! 就算正确。

#[no_mangle]
pub fn main(_args: usize, args_str: &[&str]) -> i32 {
    assert_eq!(args_str.len(),2);
    let filename = args_str[1];
    let fd = open(filename, OpenFlags::R);
    assert!(fd > 0);
    let fd = fd as usize;
    let stat: Stat = Stat::new();
    let ret = fstat(fd, &stat);
    assert_eq!(ret, 0);
    assert_eq!(stat.mode,StatMode::FILE);
    INFO!("dev: {}\nino: {}\nmode: {:?}\nnlink: {}",stat.dev,stat.ino,stat.mode,stat.nlink);
    println!("");
    close(fd);
    0
}