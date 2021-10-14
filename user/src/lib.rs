#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(asm)]

#[macro_use]
pub mod console;
mod lang_items;
pub mod syscall;


use crate::syscall::{sys_exit, sys_write,sys_yield};

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn yield_()->isize{
    sys_yield()
}
//weak弱链接，在进行链接时优先寻找bin文件下各个用户程序的入口

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    // 我们需要手动初始化.bss段，因为没有系统库或操作系统提供支持会将其初始化为0
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    unsafe {
        (start_bss as usize..end_bss as usize).for_each(|a| {
            (a as *mut u8).write_volatile(0)
        });
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
//代码编译后的汇编代码中放在一个名为 .text.entry 的代码段中
//便于将其放在 链接文件中
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("unreachable after sys_exit!");
}