///配置文件
///存放一些常量
pub const MAX_APP_NUM: usize = 10;
pub const APP_BASE_ADDRESS: usize = 0x80400000; //应用程序起始地址
pub const APP_SIZE_LIMIT: usize = 0x20000; //应用程序的空间限制
pub const USER_STACK_SIZE: usize = 4096 * 2; //用户栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; //内核栈大小
pub const BIG_STRIDE: usize = 1000;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; //内核的可分配堆大小
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BIT: usize = 12; //页大小需要12个bit位保存
pub const MEMORY_END :usize = 0x80800000;//内存的最大值

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;

#[cfg(feature = "LOG")]
pub const MINIEST_INFO: usize = 0;
