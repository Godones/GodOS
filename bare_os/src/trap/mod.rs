
pub mod context;
use crate::syscall::syscall;

use riscv::register::{
    scause::{self,Exception,Trap},
    stvec,sscratch,stval,
    sstatus::{self},
};

use crate::println;
use context::TrapFrame;
use crate::batch::run_next_app;

global_asm!(include_str!("trap.asm"));

pub fn init(){
    unsafe {
        extern "C"{
            fn _alltraps();
        }
        stvec::write(_alltraps as usize,stvec::TrapMode::Direct);
        sstatus::set_sie();//s态全局使能位
    }
    println!("++++ setup trap! ++++");
}

///根据不同类型的中断选择不同的处理
#[no_mangle]
pub fn trap_handler(tf:&mut TrapFrame)->&mut TrapFrame{
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        //系统调用
        Trap::Exception(Exception::UserEnvCall)=>{
            tf.sepc +=4;//指令地址
            tf.reg[10] = syscall(tf.reg[17],[tf.reg[10],tf.reg[11],tf.reg[12]]) as usize;

        }
        //页错误，应该是内存泄露什么的？
        Trap::Exception(Exception::StorePageFault|Exception::StoreFault)=>{
            println!("[kernel] PageFault in application, core dumped.");
            run_next_app();//下一个app

        }
        //非法指令
        Trap::Exception(Exception::IllegalInstruction)=>{
            println!("[kernel]  IllegalInstruction in application, core dumped.");
            run_next_app();
        }
        //断点中断
        Trap::Exception(Exception::Breakpoint) => {
            breakpoint_handler(&mut tf.sepc)
        }
        //s态时钟中断
        // Trap::Interrupt(Interrupt::SupervisorTimer) =>{
        //     // supertimer_handler()
        //     println!("")
        // }
        _ =>panic!("undefined trap cause: {:?}, stval: {:?}",scause.cause(),stval)
    }
    tf
}
fn breakpoint_handler(sepc:&mut usize){
    println!("Breakpoint is setted @0x{:x}",sepc);
    *sepc += 4;
}


// //S态时钟处理函数
// fn supertimer_handler(){
//     clock_next_time();
//     unsafe {
//         //修改静态变量是非常危险的操作
//         TICKS +=1;
//         if TICKS==100{
//             TICKS = 0;
//             // shutdown();
//             println!("The {} timer interrupt!",100);
//         }
//
//     }
//     //外界中断,我们并未执行完该有的操作，此处不跳到下一行指令
// }