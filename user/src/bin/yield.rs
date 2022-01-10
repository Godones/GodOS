#![no_main]
#![no_std]

use lib::{getpid, println, yield_};
#[no_mangle]
fn main() -> isize {
    println!("Hello, I am process {}.", getpid());
    for i in 0..5 {
        yield_();
        println!("Back in process {}, iteration {}.", getpid(), i);
    }
    println!("yield pass.");
    0
}
