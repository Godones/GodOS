use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::my_struct::my_ref_cell::MyRefCell;
use crate::task::processor::copy_current_task;
use crate::task::{add_task, block_current_run_next, suspend_current_run_next, TaskControlBlock};
///！ 互斥锁实现
/// 可在多线程使用
pub trait Mutex:Send+Sync{
    fn lock(&self);
    fn unlock(&self);
}
/// 互斥锁
/// 作用：如果所需资源已经被占用，则会加入等待队列
pub struct MutexBlock{
    inner:MyRefCell<MutexBlockInner>
}
pub struct MutexBlockInner{
    locked:bool,//是否处于加锁状态
    wait_queue:VecDeque<Arc<TaskControlBlock>>//等待队列
}

impl MutexBlock {
    pub fn new()->Self{
        Self{
            inner:MyRefCell::new(
                MutexBlockInner{
                    locked:false,
                    wait_queue:VecDeque::new(),
                }
            )
        }
    }
}
impl Mutex for MutexBlock {
    fn lock(&self) {
        let mut inner = self.inner.get_mut();
        if inner.locked{
            //如果已经被锁上，则加入等待队列
            inner.wait_queue.push_back(copy_current_task().unwrap());
            drop(inner);
            block_current_run_next();//暂停当前线程运行其它线程
        }
        else {
            inner.locked = true;
        }
    }
    fn unlock(&self) {
        let mut inner = self.inner.get_mut();
        assert_eq!(inner.locked,true);
        if let Some(task) = inner.wait_queue.pop_front() {
            add_task(task)//释放队列的等待线程
        }
        else {
            inner.locked = false;
        }
    }
}

/// 自旋锁
/// 作用：一直忙等待直到得到所需资源
pub struct MutexSpin{
    locked:MyRefCell<bool>,
}

impl MutexSpin {
    pub fn new()->Self{
        Self{
            locked:MyRefCell::new(false)
        }
    }
}

impl Mutex for MutexSpin{
    fn lock(&self) {
        // DEBUG!("try lock spin");
        loop {
            let mut locked = self.locked.get_mut();
            if *locked{
                //如果已经加锁
                drop(locked);
                suspend_current_run_next();
                continue
            }
            else {
                *locked = true;
                return;
            }
        }

    }
    fn unlock(&self) {
        let mut inner = self.locked.get_mut();
        *inner = false;
        // DEBUG!("release spin");
    }
}