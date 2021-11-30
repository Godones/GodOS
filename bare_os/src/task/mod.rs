/// 为了更好地完成任务上下文切换，需要对任务处于什么状态做明确划分
///任务的运行状态：未初始化->准备执行->正在执行->已退出
pub mod context;
mod manager;
mod pid;
pub mod process;
mod switch;
mod task;

use crate::{task::context::TaskContext};
use crate::task::task::{TaskControlBlock, TaskStatus};
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use manager::add_task;
pub use process::{current_user_token,current_trap_cx_ptr,take_current_task,run,schedule};
use crate::loader::get_data_by_name;

lazy_static! {
    pub static ref INITPROC:Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(get_data_by_name("initproc").unwrap()));
    //初始化初始进程
}

pub fn add_initproc() {
    //将初始进程加入任务管理器中
    add_task(INITPROC.clone());
}

pub fn suspend_current_run_next() {
    // DEBUG!("[kernel] suspend_run_next");
    //将当前任务变成暂停状态
    //将cpu执行的任务剥夺
    let task = take_current_task().unwrap();
    let mut task_inner = task.get_inner_access();
    //改变其状态
    task_inner.task_status = TaskStatus::Ready;
    //获取任务上下文指针
    let task_cx_ptr = &mut task_inner.task_cx_ptr as *mut TaskContext;
    drop(task_inner); //释放引用
    // DEBUG!("[kernel] suspend_last pid :{}",task.get_pid());
    add_task(task); //加入到任务管理器中
                    //进行任务切换
    schedule(task_cx_ptr);
}

pub fn exit_current_run_next(exit_code:isize){
    //终止当前任务运行下一个任务
    //获得当前cpu执行的任务
    let current_task = take_current_task().unwrap();
    let mut current_task_inner = current_task.get_inner_access();
    //标记僵尸进程
    current_task_inner.task_status = TaskStatus::Zombie;
    //保存返回码
    current_task_inner.exit_code = exit_code;
    {
        let mut init_proc_inner = INITPROC.get_inner_access();
        for child in current_task_inner.children.iter(){
            //挂载到初始进程的孩子上
            child.get_inner_access().parent = Some(Arc::downgrade(child));
            init_proc_inner.children.push(child.clone());
        }
    }
    //自动解除引用
    current_task_inner.children.clear();//释放子进程的引用计数
    current_task_inner.memory_set.clear_area_data();//提前释放掉地址空间
    drop(current_task_inner);
    drop(current_task);//去掉当前进程的引用，相当于销毁了进程
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut TaskContext);//重新调度
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
