#![no_std]
#![no_main]
#![feature(llvm_asm)]
mod panic;
// mod sbi;
// mod console;
mod print;
use core::fmt::Write;

#[no_mangle]
extern "C" fn _start(){
    // loop{};
    // print::sys_write(1,&['G' as u8,'o' as u8,'d' as u8, 'O' as u8,'S' as u8,'\n' as u8]);
    println!("Hello world");
    print::sys_exit(9);//退出码为9
}