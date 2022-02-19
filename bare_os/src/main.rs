#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]
#![feature(const_mut_refs)]
#[macro_use]
pub mod panic;
mod config;
mod driver;
mod file;
mod mm;
mod my_struct;
mod sbi;
mod sync;
mod syscall;
mod system_allocator;
mod task;
mod tests;
pub mod timer;
mod trap;
extern crate alloc;
extern crate easyfs;

use crate::config::KERNEL_HEAP_SIZE;
use crate::driver::gpu;
use crate::file::list_apps;
use crate::sbi::shutdown;
use crate::system_allocator::buddy::{find_last_min_pow2, test_alloc_dealloc, test_buddy};
use crate::task::add_initproc;
use buddy_system_allocator::linked_list::ListNode;
use core::arch::global_asm;
use core::mem::size_of;
use core::ptr::null_mut;

global_asm!(include_str!("entry.asm"));
// global_asm!(include_str!("link_app.S"));

// include_str! 宏将同目录下的汇编代码 entry.asm 转化为字符串并通过
// global_asm! 宏嵌入到代码中。
fn clear_bss() {
    // 我们需要手动初始化.bss段，因为没有系统库或操作系统提供支持会将其初始化为0
    unsafe {
        extern "C" {
            fn sbss();
            fn ebss();
        }
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[no_mangle]
extern "C" fn rust_main() -> ! {
    clear_bss();
    INFO!("[kernel] Godone OS");

    // test
    {
        // color_output_test();
        crate::system_allocator::heap_test();
        // crate::mm::frame_allocator::frame_test();
    }
    // trap初始化，设置stvec的入口地址
    mm::init();
    mm::remap_test(); //测试内核映射的正确性
                      //运行程序

    trap::init();
    println!("set trap over");

    // gpu();
    list_apps();
    add_initproc();
    timer::enable_timer_interrupt(); //使能位
    timer::set_next_timetrigger();
    // INFO!("Run process......");
    task::run();
    // test_alloc_dealloc();
    // test_buddy();
    shutdown();
}
