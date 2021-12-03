#![no_main]
#![no_std]
#![feature(asm)]
#![allow(non_snake_case)]
#[macro_use]
extern crate lib;
use lib::{get_time, yield_, Time};

#[no_mangle]
fn main() -> i32 {
    let mut time = Time::new();
    get_time(&mut time);
    let wait_for = time.s * 1000 + 3000; //等待3s+..
    println!("wait_for: {}", wait_for);
    loop {
        get_time(&mut time);
        if time.s * 1000 > wait_for {
            break;
        }
        yield_();
    }
    println!("Test sleep OK!");
    0
}
