
pub mod context;
use crate::syscall::syscall;

use riscv::register::{
    scause::{self,Exception,Trap},
    stvec,stval,sstatus
};

use crate::{println,ERROR};
use context::TrapFrame;
use crate::batch::run_next_app;

global_asm!(include_str!("trap.asm"));

/// 批处理操作系统初始化
/// 需要设置好stvec寄存器指向正确的Trap处理入口点
/// 当trap发生时，cpu根据stvec的地址执行相应的处理，这里将其
/// 设置为_alltraps, 则cpu会保存上下文
/// 在_alltraps 的最后会调用trap_handler函数对不同的错误进行处理
/// 处理完成 后在调用_restore恢复上下文
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
/// 在trap.asm中我们将x10-a0的值设置为sp的值，即内核栈地址
/// 此时我们以及按照TrapFrame的布局设置好了各个寄存器的值
/// tf 就是trap里面保存好的东西
/// 而函数原样返回传入的tf，因此a0寄存器仍然是进入函数时的值
#[no_mangle]
pub fn trap_handler(tf:&mut TrapFrame)->&mut TrapFrame{
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        //系统调用
        Trap::Exception(Exception::UserEnvCall)=>{
            //指令地址 需要跳转到下一句执行，否则就处于死循环中
            // println!("[kernel] UserEnvCall in application.");
            tf.sepc +=4;
            tf.reg[10] = syscall(tf.reg[17],[tf.reg[10],tf.reg[11],tf.reg[12]]) as usize;
        }
        //页错误，应该是内存泄露什么的？
        Trap::Exception(Exception::StorePageFault|Exception::StoreFault)=>{
            println!("[kernel] PageFault in application, core dumped.");
            run_next_app();//下一个app

        }
        //非法指令
        Trap::Exception(Exception::IllegalInstruction)=>{
            ERROR!("[kernel]  IllegalInstruction in application, core dumped.");
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
        _ =>{
            panic!("undefined trap cause: {:?}, stval: {:?}",scause.cause(),stval)
        }
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