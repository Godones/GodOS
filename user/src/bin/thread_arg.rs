#![no_std]
#![no_main]


extern crate alloc;

use lib::{thread_create, waittid, exit, print, println};
use alloc::vec::Vec;

struct Argument {
    pub ch: char,
    pub rc: i32,
}

fn thread_print(arg: *const Argument) -> ! {
    let arg = unsafe { &*arg };
    for _ in 0..100 { print!("{}", arg.ch); }
    exit(arg.rc)
}

#[no_mangle]
pub fn main() -> i32 {
    let mut v = Vec::new();
    let args = [
        Argument { ch: 'a', rc: 1, },
        Argument { ch: 'b', rc: 2, },
        Argument { ch: 'c', rc: 3, },
    ];
    for i in 0..3 {
        v.push(thread_create(thread_print as usize, &args[i] as *const _ as usize));
    }
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("main thread exited.");
    0
}