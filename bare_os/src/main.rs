#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![allow(dead_code)]

#[macro_use]
mod panic;
mod config;
mod loader;
mod sbi;
mod syscall;
mod task;
mod tests;
mod trap;
mod timer;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));
// include_str! 宏将同目录下的汇编代码 entry.asm 转化为字符串并通过
// global_asm! 宏嵌入到代码中。

fn clear_bss() {
    // 我们需要手动初始化.bss段，因为没有系统库或操作系统提供支持会将其初始化为0
    unsafe {
        extern "C" {
            fn sbss();
            fn ebss();
        }
        (sbss as usize..ebss as usize).for_each(|a| (a as *mut u8).write_volatile(0));
    }
}

#[no_mangle]
extern "C" fn rust_main() -> ! {
    clear_bss();
    INFO!("[kernel] Godone OS");
    // color_output_test();
    //trap初始化，设置stvec的入口地址
    trap::init();
    //运行程序
    loader::load_app();
    timer::enable_timer_interrupt();//使能位
    timer::set_next_timetrigger();
    task::run_first_task();
    panic!("The main_end!");

}
