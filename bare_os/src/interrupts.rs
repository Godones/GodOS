use riscv::register::{
    scause::{Exception,Trap,Interrupt},
    stvec,sscratch,
    sstatus::{self},
};

use crate::println;
use crate::contexts::TrapFrame;
use crate::timer::{clock_next_time, TICKS};

global_asm!(include_str!("trap/trap.asm"));

pub fn init(){
    unsafe {
        extern "C"{
            fn _alltraps();
        }
        sscratch::write(0);//位于内核态，将sscratch设置为0
        //在产生中断时根据 sscratch 的值是否为 0 来判断是在 S 态产生的中断还是 U 态（用户态）产生的中断
        stvec::write(_alltraps as usize,stvec::TrapMode::Direct);
        sstatus::set_sie();//s态全局使能位
    }
    println!("++++ setup interrupt! ++++");
}

///根据不同类型的中断选择不同的处理
#[no_mangle]
pub fn rust_trap(tf:&mut TrapFrame){
    match tf.scause.cause() {
        //断点中断
        Trap::Exception(Exception::Breakpoint) => breakpoint_handler(&mut tf.sepc),
        //s态时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) =>supertimer_handler(),
        _ =>panic!("undefined trap")
    }
}
fn breakpoint_handler(sepc:&mut usize){
    println!("Breakpoint is setted @0x{:x}",sepc);
    *sepc +=2;
}
//S态时钟处理函数
fn supertimer_handler(){
   clock_next_time();
    unsafe {
        //修改静态变量是非常危险的操作
        TICKS +=1;
        if TICKS==100{
            TICKS = 0;
            // shutdown();
            println!("The {} timer interrupt!",100);
        }

    }

    //外界中断,我们并未执行完该有的操作，此处不跳到下一行指令

}