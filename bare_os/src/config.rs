///配置文件
///存放一些常量
pub const MAX_APP_NUM: usize = 10;
pub const APP_BASE_ADDRESS: usize = 0x80400000; //应用程序起始地址
pub const APP_SIZE_LIMIT: usize = 0x20000; //应用程序的空间限制
pub const USER_STACK_SIZE: usize = 4096 * 2; //用户栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; //内核栈大小


// #[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;