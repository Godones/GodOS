use spin::{Mutex, MutexGuard};

/// 堆分配器的通用模块和函数

/// 由于不能在内部类型上面实现内部的trait ：?????
/// 我们需要包装一下Mutex使得我们可以使用Mutex
pub struct Locked<T> {
    inner: Mutex<T>,
}
impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }
    pub fn lock(&self) -> MutexGuard<T> {
        self.inner.lock()
    }
}

pub fn align_up(address: usize, align: usize) -> usize {
    //内存对齐
    //address:开始地址  align:对齐要求
    //比如 41 4
    let need_jump = address % align;
    if need_jump == 0 {
        address
    } else {
        address - need_jump + align
    }
}
