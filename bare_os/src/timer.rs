use riscv::register::{time,sie};
use crate::sbi::set_timer;
use crate::println;

pub static mut TICKS:usize = 0;
static TIMEBASE:u64 = 100000;//时钟频率，cpu频率的1%

pub fn init(){
    unsafe {
        TICKS = 0;
        sie::set_stimer();//设置时钟使能
    }
    // 硬件机制问题我们不能直接设置时钟中断触发间隔
    // 只能当每一次时钟中断触发时
    // 设置下一次时钟中断的触发时间
    // 设置为当前时间加上 TIMEBASE

    clock_next_time();
    println!("++++ setup timer!    ++++");
}
pub fn clock_next_time(){
    set_timer(time::read64()+&TIMEBASE);
}
