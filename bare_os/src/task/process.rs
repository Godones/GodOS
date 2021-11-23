use crate::task::context::TaskContext;
use crate::task::task::{TaskControlBlock, TaskStatus};
use crate::trap::context::TrapFrame;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::sbi::Console;
use crate::task::manager::fetch_task;
use crate::task::switch::__switch;

lazy_static! {
    static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}

pub struct Processor {
    //当前cpu执行的进程
    current: Option<Arc<TaskControlBlock>>,
    //进程切换上下文
    //这是一个特殊的进程切换上下文，用于从当前的任务管理器中选择一个任务进行执行
    idle_task_cx_ptr: TaskContext,

}

impl Processor {
    fn new() -> Self {
        Self {
            current: None,
            idle_task_cx_ptr: TaskContext::zero_init(),
        }
    }
    fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take() //移出当前任务。剥夺所有权
    }
    fn copy_current(&self) -> Option<Arc<TaskControlBlock>> {
        //获取当前执行任务的一份拷贝
        //用于fork()调用
        self.current.as_ref().map(|task| Arc::clone(task))
    }
    fn get_idle_task_cx_ptr(&self)->*mut TaskContext{
        &self.idle_task_cx_ptr as *mut _
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().take_current()
}
pub fn copy_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.lock().copy_current()
}
pub fn current_user_token() -> usize {
    //当前进程用户地址空间的satp
    copy_current_task()
        .unwrap()
        .get_inner_access()
        .get_user_token()
}
pub fn current_trap_cx_ptr() -> &'static mut TrapFrame {
    //获取当前进程的trap上下文
    copy_current_task()
        .unwrap()
        .get_inner_access()
        .get_trap_cx()
}
///idle控制流的作用是将进程切换隔离开来，这样换入换出进程时所用的栈是不一样的
/// idle控制流用于进程调度，其位于内核进程的栈上，而换入换出是在应用的内核栈进行
pub fn began_run_task(){
    //在内核初始化完成之后需要开始运行
    loop {
        let mut processor = PROCESSOR.lock();
        if let Some(task) = fetch_task(){
            //从任务管理器成功弹出一个任务
            let task_inner = task.get_inner_access();
            let next_task_cx_ptr = &task_inner.task_cx_ptr as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);//释放掉获取的引用，因为要切换进程了
            processor.current = Some(task);
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            drop(processor);//释放引用
            unsafe {
               __switch(
                   idle_task_cx_ptr,next_task_cx_ptr
               );
           }

        }
    }
}
pub fn schedule(last_task_cx_ptr:*mut TaskContext){
    //当时间片用完或者是进程自行放弃cpu时，需要切换进程
    // 这时需要切换到内核进行进程切换的idle控制流上，在
    //上面的began_run_task中，当内核开始运行第一个进程时，
    //就会在内核栈上形成自己的任务上下文，其返回时继续进行
    //循环查找下一个进程
    let processor = PROCESSOR.lock();
    let idle_task_cx_ptr = &processor.idle_task_cx_ptr as *const TaskContext;
    drop(processor);
    unsafe {
        __switch(
            last_task_cx_ptr,
            idle_task_cx_ptr,
        );
    }
}