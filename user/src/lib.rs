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
mod file;

extern crate alloc;
use crate::syscall::*;
use alloc::vec::Vec;
use bitflags::bitflags;
use syscall::{sys_getpid, sys_spawn};
use system_allocator::init;
pub use time::Time;
pub use file::{Stat,StatMode};
bitflags! {
    pub struct OpenFlags:u32 {
        const R = 0;//只读
        const W = 1<<0;//只写
        const RW = 1<<1;//读写
        const C = 1<<9;//新建
        const T = 1<<10;//打开清空
    }
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}
pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    sys_read(fd, buf)
}
pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code)
}
pub fn get_time(time: &mut Time) -> isize {
    sys_get_time(time)
}

pub fn get_time_ms()->usize{
    let mut time = Time::new();
    sys_get_time(&mut time);
    time.into()

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

pub fn exec(path: &str, args: &[*const u8]) -> isize {
    sys_exec(path, args)
}

pub fn spawn(path: &str) -> isize {
    sys_spawn(path)
}
pub fn getpid() -> isize {
    sys_getpid()
}

pub fn munmap(start: usize, len: usize) -> isize {
    sys_munmap(start, len)
}
pub fn mmap(start: usize, len: usize, port: usize) -> isize {
    sys_mmap(start, len, port)
}
pub fn pipe(pipe: &mut [usize]) -> isize {
    //创建一个管道
    sys_pipe(pipe)
}
pub fn close(fd: usize) -> isize {
    //关闭文件描述符
    sys_close(fd)
}

pub fn sleep(ms: usize) {
    //其实是get_time的包装
    let mut time = Time::new();
    get_time(&mut time);
    let wait_for = ms; //等待ms+..
    loop {
        get_time(&mut time);
        if time.s * 1000 > wait_for {
            break;
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

//读取进程邮箱内的内容
pub fn mail_read(buf: &mut [u8]) -> isize {
    sys_mail_read(buf)
}
// 往指定进程的邮箱块内写入内容
pub fn mail_write(pid: usize, buf: &mut [u8]) -> isize {
    sys_mail_write(pid, buf)
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits())
}

pub fn dup(fd: usize) -> isize {
    sys_dup(fd)
}
pub fn ls() -> isize {
    sys_ls()
}
///硬链接
pub fn link(oldpath:&str,newpath:&str)->isize{
    sys_linkat(-100, oldpath.as_ptr(), -100, newpath.as_ptr(), 0) as isize
}
///解除链接
pub fn unlink(path:&str)->isize{
    sys_unlinkat(-100,path.as_ptr(),0)
}
/// 查看文件信息
pub fn fstat(fd:usize,state:&Stat)->isize{
    sys_fstat(fd,state)
}
pub fn thread_create(entry:usize,arg:usize)->isize{
    sys_thread_create(entry,arg)
}
pub fn waittid(tid:usize)->isize{
    loop {
        match sys_waittid(tid) {
            -2 => yield_(),
            exit_code => return exit_code
        };
    }
}

pub fn mutex_blocking_create()->isize{
    sys_mutex_blocking_create()
}
pub fn mutex_lock(lock_id:usize)->isize{
    sys_mutex_lock(lock_id)
}
pub fn mutex_unlock(lock_id:usize)->isize{
    sys_mutex_unlock(lock_id)
}
pub fn mutex_create()->isize{
    sys_mutex_create()
}
pub fn semaphore_up(sem_id:usize)->isize{
    sys_semaphore_v(sem_id)
}
pub fn semaphore_down(sem_id:usize)->isize{
    sys_semaphore_p(sem_id)
}
pub fn semaphore_create(count:usize)->isize{
    sys_semaphore_create(count)
}

pub fn monitor_create()->isize{
    sys_monitor_create()
}

pub fn monitor_signal(mon_id:usize)->isize{
    sys_monitor_signal(mon_id)
}

pub fn monitor_wait(mon_id:usize,mutex_id:usize)->isize{
    sys_monitor_wait(mon_id,mutex_id)
}
/// weak弱链接，在进行链接时优先寻找bin文件下各个用户程序的入口
#[linkage = "weak"]
#[no_mangle]
fn main(_args: usize, _arg_vec: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

#[no_mangle]
#[link_section = ".text.entry"]
/// 代码编译后的汇编代码中放在一个名为 .text.entry 的代码段中
/// 便于将其放在链接文件中
pub extern "C" fn _start(args: usize, arg_vec_base: usize) -> ! {
    //args: 参数数量
    //args_vec: 参数起始地址
    init();
    //在真正开始执行应用程序前需要解析命令行的参数用来使用
    let mut args_str: Vec<&'static str> = Vec::new();
    for i in 0..args {
        let args_str_start = unsafe {
            ((arg_vec_base + i * core::mem::size_of::<usize>()) as *const usize).read_volatile()
        };
        let len = (0usize..)
            .find(|index| unsafe { ((args_str_start + *index) as *const u8).read_volatile() == 0 })
            .unwrap();
        args_str.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(args_str_start as *const u8, len)
            })
            .unwrap(),
        );
    }
    exit(main(args, args_str.as_slice()));
}
