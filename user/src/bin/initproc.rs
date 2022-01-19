#![no_main]
#![no_std]
#![feature(asm)]
#![allow(non_snake_case)]

use lib::{exec, fork, getpid, wait};
use lib::{println, yield_};

#[no_mangle]
fn main() -> isize {
    println!("[user] goto the initproc");
    if fork() == 0 {
        println!("[user] This child process,pid :{}", getpid());
        let args = "cat";
        exec("user_shell\0", &[0 as *const u8]);
    } else {
        println!("[user] This is father process,pid: {}", getpid());
        loop {
            let mut exit_code: i32 = 0;
            //初始进程为根进程，需要等待其它进程任意一个子进程结束
            let pid = wait(&mut exit_code);
            match pid {
                -1 => {
                    println!("[user] There is no child process");
                    yield_();
                    continue;
                } //没有子进程就让出cpu
                _ => {
                    //存在一个子进程结束
                    println!(
                        "[user] init_proc free one child process pid:{} exit_code:{}!",
                        pid, exit_code
                    );
                }
            }
        }
    }
    0
}
