#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use lib::exit;
use lib::{println, semaphore_create, semaphore_down, semaphore_up};
use lib::{sleep, thread_create, waittid};

const SEM_SYNC: usize = 0;

unsafe fn first() -> ! {
    sleep(10);
    println!("First work and wakeup Second");
    semaphore_up(SEM_SYNC);
    exit(0)
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait first");
    semaphore_down(SEM_SYNC);
    println!("Second can work now");
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    // create semaphores
    assert_eq!(semaphore_create(0) as usize, SEM_SYNC);
    // create threads
    let mut threads = Vec::new();
    threads.push(thread_create(first as usize, 0));
    threads.push(thread_create(second as usize, 0));
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("sync_sem passed!");
    0
}
