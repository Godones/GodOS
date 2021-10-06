#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
mod panic;
// mod print;
mod sbi;

use sbi::{shutdown};

global_asm!(include_str!("entry.asm"));
// include_str! 宏将同目录下的汇编代码 entry.asm 转化为字符串并通过
// global_asm! 宏嵌入到代码中。


fn clear_bss(){
    // 我们需要手动初始化.bss段，因为没有系统库或操作系统提供支持会将其初始化为0
    unsafe {
        extern "C"{
            fn sbss();
            fn ebss();
        }
        (sbss as usize..ebss as usize).for_each(|a|{
            unsafe {(a as *mut u8).write_volatile(0)}
        });
    }
}

#[no_mangle]
extern "C" fn rust_main()->!{
    clear_bss();
    println!("Godone's OS");
    println!("It's so nice");
    println!("{x:#?}",x = 1);

    panic!("Stop");
}
