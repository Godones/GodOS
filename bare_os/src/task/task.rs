#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    Uninit,  //未初始化
    Ready,   //准备执行
    Running, //正在执行
    Exited,  //已经退出
}
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_cx_ptr: usize, //任务上下文栈顶地址
    pub task_status: TaskStatus,
}

impl TaskControlBlock {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize //指向指针的指针
    }
}
