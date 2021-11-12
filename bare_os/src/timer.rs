use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
/// 时钟中断 寄存器mtime中保存了自处理器上电后cpu经过了多少时钟周期
/// mtimecmp寄存器保存的是mtime的阈值，当超过阈值会发生一个时钟中断
use riscv::register::{sie, time};
pub fn get_time() -> usize {
    //获得当前mtime的时钟周期数
    time::read()
}
pub fn get_costtime() -> usize {
    //以s为单位返回cpu运行时间
    time::read() / (CLOCK_FREQ / 1000)
}
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
pub fn set_next_timetrigger() {
    //设置10ms产生一个中断
    set_timer(get_time() + CLOCK_FREQ / 50);
}
