#![no_std]
#![no_main]


use lib::{fork, exec, wait};
use lib::println;

#[no_mangle]
pub fn main() -> i32 {
    for i in 0..1000 {
        if fork() == 0 {
            exec("pip_large_test\0", &[0 as *const u8]);
        } else {
            let mut _unused: i32 = 0;
            wait(&mut _unused);
            println!("Iter {} OK.", i);
        }
    }
    0
}