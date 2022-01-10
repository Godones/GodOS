#![no_std]
#![no_main]

use lib::println;



fn f(d: usize) {
    println!("d = {}", d);
    f(d + 1);
}

#[no_mangle]
pub fn main() -> i32 {
    println!("It should trigger segmentation fault!");
    f(0);
    0
}
