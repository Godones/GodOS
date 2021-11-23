pub mod context;
mod manager;
mod pid;
mod process;
mod switch;
mod task;

impl TaskManager {
    fn get_current_token(&self) -> usize {
        //获取用户地址空间的satp
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow().tasks[current_task].get_user_token()
    }
    fn get_trap_cx(&self) -> &'static mut TrapFrame {
        //获取用户trap上下文所在位置
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].get_trap_cx()
    }
    fn mark_current_suspended(&self) {
        //将当前任务变成暂停状态
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        // 退出当前任务
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].task_status = TaskStatus::Exited;
    }
    fn set_priority(&self, priority: usize) -> isize {
        //设置优先级就等价于更改增长量
        let mut inner = self.inner.borrow_mut();
        let current_task = inner.current_task;
        inner.tasks[current_task].pass = BIG_STRIDE / priority;

        priority as isize
    }

    fn stride(&self) -> Option<usize> {
        //stride调度算法
        let mut miniest = usize::MAX;
        let mut index = 0;
        for i in 0..self.num_app {
            if self.inner.borrow().tasks[i].stride < miniest
                && self.inner.borrow().tasks[i].task_status == TaskStatus::Ready
            {
                miniest = self.inner.borrow().tasks[i].stride;
                index = i;
            }
        }
        // DEBUG!("[kernel debug] {} {}",miniest,index);
        Some(index)
    }
}
