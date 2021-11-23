use task::TaskControlBlock;
/// 为了更好地完成任务上下文切换，需要对任务处于什么状态做明确划分
///任务的运行状态：未初始化->准备执行->正在执行->已退出
pub mod context;
mod manager;
mod pid;
pub mod process;
mod switch;
mod task;

use crate::task::context::TaskContext;
use crate::task::manager::add_task;
use crate::task::process::{schedule};
use crate::task::task::TaskStatus;
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use process::{current_user_token,current_trap_cx_ptr,take_current_task};
lazy_static! {
    pub static ref INITPROC:Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new("initproc"));
    //初始化初始进程
}

pub fn add_initproc() {
    //将初始进程加入任务管理器中
    add_task(INITPROC.clone());
}

pub fn mark_current_suspended() {
    //将当前任务变成暂停状态
    //将cpu执行的任务剥夺
    let task = take_current_task().unwrap();
    let mut task_inner = task.get_inner_access();
    //改变其状态
    task_inner.task_status = TaskStatus::Ready;
    //获取任务上下文指针
    let task_cx_ptr = &mut task_inner.task_cx_ptr as *mut TaskContext;
    drop(task_inner); //释放引用
    add_task(task); //加入到任务管理器中
                    //进行任务切换
    schedule(task_cx_ptr);
}

fn mark_current_exited() {
    // 退出当前任务
}
// fn set_priority( priority: usize) -> isize {
//     //设置优先级就等价于更改增长量
//     let task =
//
//     inner.tasks[current_task].pass = BIG_STRIDE / priority;
//
//     priority as isize
// }

fn stride() -> Option<usize> {
    //stride调度算法
    todo!("完成基于特权级的调度算法")
}
