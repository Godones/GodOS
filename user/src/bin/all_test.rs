#![no_std]
#![no_main]


static TESTS: &[&str] = &[
    "exit\0",
    "rich_text\0",
    "forktest\0",
    "forktest2\0",
    "forktest1\0",
    "03sleep\0",
    "stackoverflow\0",
    "yield\0",
];

use lib::{exec, fork, println, wait_pid};

#[no_mangle]
pub fn main() -> i32 {
    for test in TESTS {
        println!("Usertests: Running {}", test);
        let pid = fork();
        if pid == 0 {
            exec(*test, &[0 as *const u8]);
            panic!("unreachable!");
        } else {
            let mut exit_code: i32 = Default::default();
            let wait_pid = wait_pid(pid as usize, &mut exit_code);
            assert_eq!(pid, wait_pid);
            println!("\x1b[32mUsertests: Test {} in Process {} exited with code {}\x1b[0m", test, pid, exit_code);
        }
    }
    println!("Usertests passed!");
    0
}