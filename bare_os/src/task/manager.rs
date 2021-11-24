use crate::task::task::TaskControlBlock;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{INFO, println};

pub struct TaskManager {
    //进程管理器，负责管理所有的进程
    //使用双端队列和引用计数进行管理，如果不将任务控制块移到
    //堆上进行存储，任务管理器只保留指针，那么在移动任务控制块时会
    //带来性能损耗，使用引用计数也方便操作
    task_ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            task_ready_queue: VecDeque::new(),
        }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        //添加一个进程控制块
        self.task_ready_queue.push_back(task)
    }
    pub fn pop(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.task_ready_queue.pop_front() //FIFO，先进先出调度
    }
}
lazy_static! {
    static ref TASKMANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASKMANAGER.lock().add(task);
    INFO!("[kernel] There is {} task",TASKMANAGER.lock().task_ready_queue.len());
}
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    let next = TASKMANAGER.lock().pop();
    INFO!("[kernel] Get the pid: {} task",next.as_ref().unwrap().get_pid());
    next
}
