#![no_main]
#![no_std]
#![feature(llvm_asm)]

#[macro_use]
extern crate lib;


#[no_mangle]
fn main() -> i32 {
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    unsafe { (0x0 as *mut u8).write_volatile(0); }
    0
}