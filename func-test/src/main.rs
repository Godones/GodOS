#![feature(const_mut_refs)]
mod common;
mod linked_list;
mod my_buddy;
mod rcore_buddy;
mod config;

extern crate alloc;

fn main() {
    let mut time = 0;
    for _ in 0..10{
        unsafe {
            time +=rcore_buddy::test();
        }
    }
    println!("rcore_cost:{}",time/10);
    time = 0;
    for _ in 0..10{
        unsafe {
            time +=my_buddy::test_buddy();
        }
    }
    println!("my_buddy_cost:{}",time/10);
}