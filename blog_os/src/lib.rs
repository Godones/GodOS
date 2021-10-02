#![cfg_attr(test,no_main)]
#![no_std]
#![feature(custom_test_frameworks)]//启用自定义测试框架
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]//重定义test生成的主函数main名称

use core::panic::PanicInfo;
pub mod serial;
pub mod vga_buffer;

#[repr(u32)]
pub enum QemuExitCode{
    Success = 0x10,
    Failed = 0x11
}

// #[cfg(test)]//#[cfg(tests)]属性保证它只会出现在测试中
pub fn test_runner(tests:&[&dyn Fn()]){
    serial_println!("Running {} tests",tests.len());
    for test in tests{
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_asserting(){
    serial_print!("trivial assertion... ");
    assert_eq!(1,1);
    serial_println!("[ok]");
}

pub fn exit_qemu(exit_code:QemuExitCode){
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);//创建端口
        port.write(exit_code as u32);
    }
}

pub fn test_panic_handler(info:&PanicInfo)->!{
    serial_println!("[Failed]");
    serial_println!("Error: {}",info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}
