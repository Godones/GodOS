#![no_main]
#![no_std]
#![feature(asm)]
#![allow(non_snake_case)]
#[macro_use]
extern crate lib;
use lib::{get_time, Time, yield_};

#[no_mangle]
fn main() -> i32 {
    let mut time = Time::new();
    get_time(&time);
    let wait_for = time.s*1000 + 3000;
    loop {
        get_time(&time);
        if time.s * 1000 < wait_for {
            break
        }
        yield_();
    }

    println!("Test sleep OK!");
    0
}
