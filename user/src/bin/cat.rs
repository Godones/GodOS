#![no_main]
#![no_std]
#![allow(non_snake_case)]

extern crate alloc;
use alloc::string::String;
use lib::{close, open, OpenFlags, println, read, write};

#[no_mangle]
fn main(args:usize,args_str:&[&str])->i32 {
    assert_eq!(args, 2);//第一个参数是cat 第二个参数是文件名
    println!("Show the file info...");
    let fd = open(args_str[1],OpenFlags::R);
    if fd==-1 {
        panic!("Cannot open the file!");
    }
    let fd = fd as usize;
    let mut buf = [0u8;16];
    let mut str = String::new();
    loop {
        let len = read(fd,&mut buf) as usize;
        // println!("len: {}",len);
        if len==0 { break;}
        str.push_str(core::str::from_utf8(&buf[..len]).unwrap());
    }
    println!("{}",str);
    close(fd);
    0
}