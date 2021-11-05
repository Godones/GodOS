pub mod context;
use crate::syscall::syscall;
use crate::timer::set_next_timetrigger;

use crate::config::TRAMPLINE;
use crate::task::{current_trap_cx, current_user_token, suspend_current_run_next};
use crate::{println, ERROR};
use context::TrapFrame;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sstatus, stval, stvec,
};

global_asm!(include_str!("trap.asm"));

/// 批处理操作系统初始化
/// 需要设置好stvec寄存器指向正确的Trap处理入口点
/// 当trap发生时，cpu根据stvec的地址执行相应的处理，这里将其
/// 设置为_alltraps, 则cpu会保存上下文
/// 在_alltraps 的最后会调用trap_handler函数对不同的错误进行处理
/// 处理完成 后在调用_restore恢复上下文
pub fn init() {
    unsafe {
        extern "C" {
            fn _alltraps();
        }
        stvec::write(_alltraps as usize, stvec::TrapMode::Direct);
        sstatus::set_sie(); //s态全局使能位
    }
    println!("++++ setup trap! ++++");
}
pub fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, stvec::TrapMode::Direct);
    }
}
#[no_mangle]
fn trap_from_kernel() -> ! {
    panic!("[kernel] trap from kernel")
}
pub fn set_user_trap_entry() {
    //设置用户态trap处理入口
    unsafe {
        stvec::write(TRAMPLINE, stvec::TrapMode::Direct);
    }
}
#[no_mangle]
pub fn trap_return() -> ! {
    //返回用户态继续执行
    set_user_trap_entry();
    let trap_cx = current_trap_cx();
    let user_satp = current_user_token();
    extern "C" {
        fn _alltraps();
        fn _restore();
    }
    let restore_viraddress = (_restore as usize - _alltraps as usize) + TRAMPLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_viraddress}",
            restore_viraddress = in(reg) restore_viraddress,
            in("a0") trap_cx,
            in("a1") user_satp,
            options(nostack)
        );
    }
    panic!("")
}
///根据不同类型的中断选择不同的处理
/// 在trap.asm中我们将x10=a0的值设置为sp的值，即内核栈地址
/// 此时我们已经按照TrapFrame的布局设置好了各个寄存器的值
/// tf 就是trap里面保存好的东西
///
#[no_mangle]
pub fn trap_handler() -> ! {
    //在进入内核后，会有可能再次触发中断或者其它异常
    //此时我们直接panic而不做其它处理
    set_kernel_trap_entry();
    let tf = current_trap_cx();

    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        //系统调用
        Trap::Exception(Exception::UserEnvCall) => {
            //指令地址 需要跳转到下一句执行，否则就处于死循环中
            // println!("[kernel] UserEnvCall in application.");
            tf.sepc += 4;
            tf.reg[10] = syscall(tf.reg[17], [tf.reg[10], tf.reg[11], tf.reg[12]]) as usize;
        }
        //页错误，应该是内存泄露什么的？
        Trap::Exception(Exception::StorePageFault | Exception::StoreFault) => {
            ERROR!("[kernel] PageFault in application, core dumped.");
            panic!("StorePageFault");
        }
        //非法指令
        Trap::Exception(Exception::IllegalInstruction) => {
            ERROR!("[kernel]  IllegalInstruction in application, core dumped.");
            panic!();
        }
        //断点中断
        Trap::Exception(Exception::Breakpoint) => breakpoint_handler(&mut tf.sepc),
        //s态时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            supertimer_handler();
        }
        _ => {
            panic!(
                "undefined trap cause: {:?}, stval: {:?}",
                scause.cause(),
                stval
            )
        }
    }
    trap_return()
}
fn breakpoint_handler(sepc: &mut usize) {
    println!("Breakpoint is setted @0x{:x}", sepc);
    *sepc += 4;
}

//S态时钟处理函数
fn supertimer_handler() {
    set_next_timetrigger();
    suspend_current_run_next();
}
