use crate::mm::address::PhysPageNum;
use crate::my_struct::my_ref_cell::MyRefCell;
use crate::task::context::TaskContext;
use crate::task::id::{kernel_stack_alloc, KernelStack, TaskUserRes};
use crate::task::process::ProcessControlBlock;
use crate::trap::context::TrapFrame;
///! 线程定义
use alloc::sync::{Arc, Weak};
use core::cell::RefMut;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    Ready,    //准备执行
    Running,  //正在执行
    Blocking, //已经退出
}
pub struct TaskControlBlock {
    //不可变数据
    pub process: Weak<ProcessControlBlock>, //所属进程
    pub kernel_stack: KernelStack,          //内核栈
    //可变数据
    inner: MyRefCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub res: Option<TaskUserRes>, //线程信息
    pub task_cx_ptr: TaskContext, //线程上下文栈顶地址的位置,位于内核空间中
    pub task_status: TaskStatus,
    pub trap_cx_ppn: PhysPageNum, //线程trap上下文所在位置
    pub exit_code: Option<i32>,   //保存退出码
}

impl TaskControlBlock {
    pub fn new(
        father_process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let user_res = TaskUserRes::new(ustack_base, father_process.clone(), alloc_user_res); //创建线程所需的资源
        let trap_cx_ppn = user_res.trap_cx_ppn(); //trap上下文所在页面
        let kstack = kernel_stack_alloc(); //声请内核栈
        let kstack_top = kstack.get_stack_top();
        Self {
            process: Arc::downgrade(&father_process),
            kernel_stack: kstack,
            inner: MyRefCell::new(TaskControlBlockInner {
                res: Some(user_res),
                task_cx_ptr: TaskContext::goto_trap_return(kstack_top),
                task_status: TaskStatus::Ready,
                trap_cx_ppn,
                exit_code: None,
            }),
        }
    }
    pub fn get_inner_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.get_inner_access();
        inner.get_user_token()
    }
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapFrame {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_task_status(&self) -> TaskStatus {
        self.task_status
    }
}
