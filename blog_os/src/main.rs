#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]//启用自定义测试框架
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]//重定义test生成的主函数main名称


use core::panic::PanicInfo;
use blog_os::println;

/// 这个函数将在 panic 时被调用
#[panic_handler]
#[cfg(not(test))]//在非测试模式下使用
fn panic(_info: &PanicInfo) -> ! {
    println!("{}",_info);
    loop {}
}

#[panic_handler]
#[cfg(test)]
fn panic(info:&PanicInfo)->!{
    blog_os::test_panic_handler(info);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("This is very nice{}","!");
    #[cfg(test)]
        test_main();
    loop {}
}
