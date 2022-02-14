///配置文件
///存放一些常量

pub const USER_STACK_SIZE: usize = 4096 * 2; //用户栈大小
pub const KERNEL_STACK_SIZE: usize = 4096 * 2; //内核栈大小

pub const BIG_STRIDE: usize = 1000; //控制一个时间片后应用的步长
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; //内核的可分配堆大小
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BIT: usize = 12; //页大小需要12个bit位保存
pub const MEMORY_END: usize = 0x8080_0000; //内存的最大值

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
pub const RING_BUFFER_SIZE: usize = 32;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
//配置外部设备的内存映射
pub const MMIO: &[(usize, usize)] = &[
    (0x10001000, 0x1000),
    // (0x10007000,0x1000),
];

pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO1: usize = 0x10007000;

#[cfg(feature = "LOG")]
pub const MINIEST_INFO: usize = 0;
