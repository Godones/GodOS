#![allow(non_snake_case)]
use crate::my_struct::my_ref_cell::MyRefCell;
use crate::task::processor::copy_current_task;
use crate::task::{add_task, block_current_run_next, TaskControlBlock};
use alloc::collections::VecDeque;
use alloc::sync::Arc;

///! 信号量实现
pub struct Semaphore {
    inner: MyRefCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    count: isize,
    wait_task: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    pub fn new(count: usize) -> Self {
        Self {
            inner: MyRefCell::new(SemaphoreInner {
                count: count as isize,
                wait_task: VecDeque::new(),
            }),
        }
    }
    pub fn P(&self) {
        let mut inner = self.inner.get_mut();
        inner.count -= 1;
        if inner.count < 0 {
            //此时被阻塞
            inner.wait_task.push_back(copy_current_task().unwrap());
            drop(inner);
            block_current_run_next();
        }
    }
    pub fn V(&self) {
        let mut inner = self.inner.get_mut();
        inner.count += 1;
        if inner.count <= 0 {
            // 有等待的线程需要激活
            assert!(inner.wait_task.len() > 0);
            add_task(inner.wait_task.pop_front().unwrap());
        }
    }
}
