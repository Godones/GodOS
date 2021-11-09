///配置文件
///存放一些常量


pub const USER_STACK_SIZE: usize = 4096 * 2; //用户栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; //内核栈大小

pub const BIG_STRIDE: usize = 1000; //控制一个时间片后应用的步长
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; //内核的可分配堆大小
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BIT: usize = 12; //页大小需要12个bit位保存
pub const MEMORY_END: usize = 0x8080_0000; //内存的最大值
pub const PAGE_SIZE_BITS: usize = 0xc; //页标志位大小

pub const TRAMPLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAMP_CONTEXT: usize = TRAMPLINE - PAGE_SIZE;
#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;

#[cfg(feature = "LOG")]
pub const MINIEST_INFO: usize = 0;

//返回应用程序在内核的内核栈位置
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let button = top - KERNEL_STACK_SIZE;
    (button, top)
}
