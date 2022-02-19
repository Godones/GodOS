#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use lib::exit;
use lib::{
    monitor_create, monitor_signal, monitor_wait, mutex_blocking_create, mutex_lock, mutex_unlock,
    println,
};
use lib::{sleep, thread_create, waittid};

static mut A: usize = 0;

const CONDVAR_ID: usize = 0;
const MUTEX_ID: usize = 0;

unsafe fn first() -> ! {
    sleep(10);
    println!("First work, Change A --> 1 and wakeup Second");
    mutex_lock(MUTEX_ID);
    A = 1;
    monitor_signal(CONDVAR_ID);
    mutex_unlock(MUTEX_ID);
    exit(0)
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait A=1");
    mutex_lock(MUTEX_ID);
    while A == 0 {
        println!("Second: A is {}", A);
        monitor_wait(CONDVAR_ID, MUTEX_ID);
    }
    mutex_unlock(MUTEX_ID);
    println!("A is {}, Second can work now", A);
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    // create condvar & mutex
    assert_eq!(monitor_create() as usize, CONDVAR_ID);
    assert_eq!(mutex_blocking_create() as usize, MUTEX_ID);
    // create threads
    let mut threads = Vec::new();
    threads.push(thread_create(first as usize, 0));
    threads.push(thread_create(second as usize, 0));
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("test_condvar passed!");
    0
}
