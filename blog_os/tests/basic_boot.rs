#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]


///! 使用集成测试
///! 所有的集成测试都是它们自己的可执行文件，并且与我们的main.rs完全独立。
///! 这也就意味着每个测试都需要定义它们自己的函数入口点。


use core::panic::PanicInfo;
#[no_mangle]
pub extern "C" fn _start()->!{
    test_main();
    loop {}
}
use blog_os::{serial_print,serial_println,println};

#[test_case]
fn basic_boot_test_println() {
    serial_print!("basic_boot_test_println... ");
    println!("test_println output");
    serial_println!("[ok]");
}

#[panic_handler]
fn panic(info:&PanicInfo)->!{
    blog_os::test_panic_handler(info);
}