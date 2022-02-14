use crate::my_struct::my_ref_cell::MyRefCell;
use crate::sync::Mutex;
use crate::task::processor::copy_current_task;
use crate::task::{add_task, block_current_run_next, TaskControlBlock};
///! 管程实现
///! 使用基于条件变量和互斥锁实现
use alloc::collections::VecDeque;
use alloc::sync::Arc;

pub struct Monitor {
    inner: MyRefCell<MonitorInner>,
}

pub struct MonitorInner {
    wait_task: VecDeque<Arc<TaskControlBlock>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            inner: MyRefCell::new(MonitorInner {
                wait_task: VecDeque::new(),
            }),
        }
    }
    pub fn wait(&self, mutex: Arc<dyn Mutex>) {
        mutex.unlock(); //先解锁
        let mut inner = self.inner.get_mut();
        inner.wait_task.push_back(copy_current_task().unwrap());
        drop(inner);
        block_current_run_next(); //切换到别的任务
        mutex.lock(); //重新加锁
    }
    pub fn signal(&self) {
        if let Some(task) = self.inner.get_mut().wait_task.pop_front() {
            add_task(task)
        }
    }
}
