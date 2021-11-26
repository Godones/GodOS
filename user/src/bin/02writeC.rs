#![no_main]
#![no_std]
#![feature(asm)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lib;
use lib::set_priority;
const LEN: usize = 100;

#[no_mangle]
fn main() -> i32 {
    assert_eq!(32,set_priority(32));
    let p = 3u64;
    let m = 998244353u64;
    let iter: usize = 20000;
    let mut s = [0u64; LEN];
    let mut cur = 0usize;
    s[cur] = 1;
    for i in 1..=iter {
        let next = if cur + 1 == LEN { 0 } else { cur + 1 };
        s[next] = s[cur] * p % m;
        cur = next;
        if i % 10000 == 0 {
            println!("power_2 [{}/{}]", i, iter);
        }
    }
    println!("{}^{} = {}", p, iter, s[cur]);
    println!("Test power_2 OK!");
    0
}
