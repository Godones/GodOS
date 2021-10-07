#![no_main]
#![no_std]
#![feature(asm)]



#[macro_use]
extern crate lib;

#[no_mangle]
fn main()->i32{
    println!("Hello God\n");
    unsafe {
        asm!("sret");
    }
    0
}