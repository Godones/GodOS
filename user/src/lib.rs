#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(asm)]
#![feature(alloc_error_handler)]

#[macro_use]
pub mod console;
mod lang_items;
pub mod syscall;
mod system_allocator;
mod time;
pub use time::Time;


use crate::syscall::*;
use syscall::{sys_getpid, sys_spawn};
use system_allocator::init;

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
pub fn get_time(time:& mut Time) -> isize {
    sys_get_time(time)
}
pub fn yield_() -> isize {
    sys_yield()
}

pub fn set_priority(priority: isize) -> isize {
    sys_set_priority(priority)
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

pub fn spawn(path:&str) ->isize{
    sys_spawn(path)
}
pub fn getpid()->isize{
    sys_getpid()
}



pub fn munmap(start:usize,len:usize)->isize{
    sys_munmap(start,len)
}
pub fn mmap(start:usize,len:usize,port:usize)->isize{
    sys_mmap(start,len,port)
}
pub fn pipe(pipe:&mut [usize])->isize{
    //创建一个管道
    sys_pipe(pipe)
}
pub fn close(fd:usize)->isize{
    //关闭文件描述符
    sys_close(fd)
}

pub fn sleep(ms:usize){
    //其实是get_time的包装
    let mut time = Time::new();
    get_time(&mut time);
    let wait_for = ms;//等待ms+..
    loop {
        get_time(&mut time);
        if time.s * 1000 > wait_for {
            break
        }
        yield_();
    }
}

/// 等待任意一个子进程结束？
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                // println!("[user] no child process");
                yield_();
            } //如果返回值是-2，说明子进程全部没有结束
            exit_pid => return exit_pid,
        }
    }
}
/// 等待一个特定的子进程结束
pub fn wait_pid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            exit_pid => return exit_pid,
        }
    }
}

//weak弱链接，在进行链接时优先寻找bin文件下各个用户程序的入口
#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

#[no_mangle]
#[link_section = ".text.entry"]
//代码编译后的汇编代码中放在一个名为 .text.entry 的代码段中
//便于将其放在链接文件中
pub extern "C" fn _start() -> ! {
    init();
    exit(main());
    panic!("unreachable after sys_exit!");
}
