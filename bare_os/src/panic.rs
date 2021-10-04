#![feature(panic_info_message)]

use core::option::Option::Some;
use core::panic::PanicInfo;
// use crate::print::sys_exit;
// use crate::println;
use crate::sbi::shutdown;
use crate::println;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some (location) = _info.location(){
        println!("Panicked at {}:{} {}",location.file(),location.line(),_info.message().unwrap());
    }
    else {
        println!("Panicked :{}",_info.message().unwrap())
    }
    shutdown();
    // sys_exit(9);
    loop {

    }
}