#![no_main]
#![no_std]
#![feature(llvm_asm)]



#[macro_use]
extern crate lib;

#[no_mangle]
fn main()->i32{
    println!("Hello God\n");
    unsafe {
        llvm_asm!("sret");
    }
    0
}