#![no_std]
#![no_main]


use lib::{open, close, read, write, OpenFlags, println};

#[no_mangle]
pub fn main() -> i32 {
    let test_str = "Hello, world!";
    let filea = "filea\0";
    let fd = open(filea, OpenFlags::C | OpenFlags::W);
    assert!(fd > 0);
    let fd = fd as usize;
    write(fd, test_str.as_bytes());//写入内容

    println!("first fd:{}",fd);
    close(fd);
    let fd = open(filea, OpenFlags::R);//打开，读取内容
    assert!(fd > 0);
    let fd = fd as usize;
    let mut buffer = [0u8; 100];
    println!("second fd:{}",fd);
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);

    assert_eq!(
        test_str,
        core::str::from_utf8(&buffer[..read_len]).unwrap(),
    );
    println!("file_test passed!");
    0
}