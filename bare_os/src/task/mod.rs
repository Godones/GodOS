use core::cell::RefCell;
use crate::task::task::TaskControlBlock;
use crate::config::*;
use lazy_static::lazy_static;
use crate::loader::{get_num_app,init_app_cx};
use task::TaskStatus;
/// 为了更好地完成任务上下文切换，需要对任务处于什么状态做明确划分
///任务的运行状态：未初始化->准备执行->正在执行->已退出
///
mod context;
mod switch;
mod task;
//管理各个任务的任务管理器
pub struct TaskManager{
    num_app:usize,//应用数量
    inner:RefCell<TaskManagerInner>
}
struct TaskManagerInner{
    current_task:usize,//当前任务
    tasks: [TaskControlBlock;MAX_APP_NUM],
}

unsafe impl Sync for TaskManager {
}
lazy_static! {
     static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock{task_cx_ptr:0,task_status:TaskStatus::UnInit},
            MAX_APP_NUM
        ];
        for i in 0..num_app{
            tasks[i].task_cx_ptr = init_app_ptr(i) as * const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
        }
        TaskManager{
            num_app,
            inner: RefCell::new(
                TaskManager{
                    tasks,
                    current_task:0

            }
        )
    }
};
}
