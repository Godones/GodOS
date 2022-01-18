use alloc::sync::Arc;
use crate::mm::KERNEL_SPACE;
use crate::task::processor::copy_current_task;
use crate::task::{add_task, TaskControlBlock};
use crate::trap::context::TrapFrame;
use crate::trap::trap_handler;

/// 创建线程
pub fn sys_thread_create(entry:usize, arg:usize) ->isize{
    let current_task = copy_current_task().unwrap();//当前线程
    let process = current_task.process.upgrade().unwrap();//获取其所在的进程
    let new_task = Arc::new(TaskControlBlock::new(
        process.clone(),
        current_task.get_inner_access().res.as_ref().unwrap().ustack_base,
        true,
    ));
    //将子线程加入调度队列中
    add_task(new_task.clone());
    //获取线程的tid
    let new_task_inner = new_task.get_inner_access();
    let res = new_task_inner.res.as_ref().unwrap();
    let tid = res.tid;
    //获取进程的线程队列
    let mut process_inner = process.get_inner_access();
    let task = &mut process_inner.task;
    while task.len()<tid+1 {
        task.push(None)
    }
    task[tid] = Some(Arc::clone(&new_task));//加入队列中
    //修改子线程的trap上下文
    let new_task_trap_cx = new_task_inner.get_trap_cx();
    *new_task_trap_cx = TrapFrame::app_into_context(
        entry,
        res.ustack_top(),
        KERNEL_SPACE.lock().token(),
        new_task.kernel_stack.get_stack_top(),
        trap_handler as usize
    );
    (*new_task_trap_cx).reg[10] = arg;//传第一个参数
    tid as isize
}

pub fn sys_gettid()->isize{
    copy_current_task().unwrap().get_inner_access().res.as_ref().unwrap().tid as isize
}
pub fn sys_waittid(tid:usize)->i32{
    let task = copy_current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    let task_inner = task.get_inner_access();
    let mut process_inner = process.get_inner_access();
    if task_inner.res.as_ref().unwrap().tid == tid{
        // 当前线程
        return -2;
    }
    let mut exit_code :Option<i32> = None;
    if let Some(wait_task) = process_inner.task[tid].as_ref(){
        if let Some(wait_exit_code)  = wait_task.get_inner_access().exit_code {
            exit_code = Some(wait_exit_code);
        }
    }
    else {
        //线程不存在
        return -1;
    }
    if let Some(exit_code) = exit_code {
        process_inner.task[tid] = None;
        exit_code
    } else {
        //线程依然存在
        -2
    }
}
