#![no_main]
#![no_std]
#![feature(asm)]
#![allow(non_snake_case)]
#[macro_use]
extern crate lib;
use lib::{get_time, yield_};

#[no_mangle]
fn main() -> i32 {
    let current_timer = get_time();
    // println!("current_time:{}",current_timer);
    let wait_for = current_timer + 3000;
    while get_time()<wait_for{
        yield_();
    }
    println!("Test sleep OK!");
    0
}
