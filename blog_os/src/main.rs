#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;


static HELLO: &[u8] = b"Hello World!";
/// 这个函数将在 panic 时被调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {

    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_buffer::print_something();
    loop {}
}
