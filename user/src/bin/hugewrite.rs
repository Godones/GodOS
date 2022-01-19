#![no_std]
#![no_main]

use lib::{OpenFlags, open, close, write, get_time};
use lib::{println, Time};

#[no_mangle]
pub fn main() -> i32 {
    let mut buffer = [0u8; 1024]; // 1KiB
    for i in 0..buffer.len() {
        buffer[i] = i as u8;
    }
    let f = open("testf\0", OpenFlags::C | OpenFlags::W);
    if f < 0 {
        panic!("Open test file failed!");
    }
    let f = f as usize;

    // INFO!("fd:{}",f);
    let mut start = Time::new();
    get_time(&mut start);

    let size_mb = 1usize;
    for _ in 0..1024 * size_mb {
        write(f, &buffer);
    }
    // INFO!("write");
    close(f);
    let  end_time = Time::new();
    let cost = start - end_time;
    let speed_kbs = size_mb * 1000000 / cost;
    println!(
        "{}MiB written, time cost = {}ms, write speed = {}KiB/s",
        size_mb, cost, speed_kbs
    );
    0
}
