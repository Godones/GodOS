#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    Uninit,  //未初始化
    Ready,   //准备执行
    Running, //正在执行
    Exited,  //已经退出
}
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_cx_ptr: usize, //任务上下文栈顶地址的应用
    pub task_status: TaskStatus,
    pub stride:usize, //已走步长
    pub pass:usize, //每一步的步长，只与特权级相关
}

impl TaskControlBlock {
    //返回指向task栈顶的指针 -> task_cx_ptr 里面存的是task上下文的地址的地址
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize //指向指针的指针
    }
}
