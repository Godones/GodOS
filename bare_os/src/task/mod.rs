/// 为了更好地完成任务上下文切换，需要对任务处于什么状态做明确划分
///任务的运行状态：未初始化->准备执行->正在执行->已退出
pub mod context;
mod manager;
mod id;
pub mod processor;
mod switch;
mod task;
mod process;

use crate::task::context::TaskContext;
use crate::task::task::{TaskStatus};
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use task::TaskControlBlock;
pub use manager::add_task;
pub use processor::{current_trap_cx_ptr, current_user_token, run, schedule, take_current_task};
use crate::file::open_file;
use crate::file::OpenFlags;
use crate::task::process::ProcessControlBlock;

lazy_static! {
    pub static ref INITPROC:Arc<ProcessControlBlock> = {
        let node = open_file("initproc",OpenFlags::R).unwrap();
        let data = node.read_all();
        ProcessControlBlock::new(data.as_slice())
    };
    //初始化初始进程
}

pub fn add_initproc() {
    //将初始进程加入任务管理器中
    let _initproc = INITPROC.clone();
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
pub fn block_current_run_next(){
    let task = take_current_task().unwrap();
    let mut task_inner = task.get_inner_access();
    let task_cx_ptr = &mut task_inner.task_cx_ptr as *mut TaskContext;
    task_inner.task_status = TaskStatus::Blocking;
    drop(task_inner);
    schedule(task_cx_ptr);
}
pub fn exit_current_run_next(exit_code: i32) {
    //终止当前任务运行下一个任务
    //获得当前cpu执行的任务
    let current_task = take_current_task().unwrap();
    let mut current_task_inner = current_task.get_inner_access();
    let process = current_task.process.upgrade().unwrap();
    let tid = current_task_inner.res.as_ref().unwrap().tid;//线程标识符

    //保存返回码
    current_task_inner.res = None;//子线程资源回收
    current_task_inner.exit_code = Some(exit_code);
    drop(current_task_inner);
    drop(current_task);

    //如果是主线程发出此系统调用
    if tid==0 {
        let mut process_inner = process.get_inner_access();
        process_inner.is_zombie = true;//僵尸进程
        process_inner.exit_code = exit_code;
        {
            //将子进程全部挂载到初始进程上面
            let mut init_proc_inner = INITPROC.get_inner_access();
            for child in process_inner.children.iter() {
                //挂载到初始进程的孩子上
                child.get_inner_access().parent = Some(Arc::downgrade(&INITPROC));
                init_proc_inner.children.push(child.clone());
            }
        }
        //回收所有子线程的资源
        for task in process_inner.task.iter().filter(|x|x.is_some()){
            let task  = task.as_ref().unwrap();
            let mut task_inner= task.get_inner_access();
            task_inner.res = None;
        }
        process_inner.children.clear();//清空所有的子进程
        process_inner.memory_set.clear_area_data();
    }
    //自动解除引用
    drop(process);
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut TaskContext); //重新调度
}
fn set_priority( _priority: usize) -> isize {
    //设置优先级就等价于更改增长量
    todo!("设置进程优先级")
}

fn stride() -> Option<usize> {
    //stride调度算法
    todo!("完成基于特权级的调度算法")
}
