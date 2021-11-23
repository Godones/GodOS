use crate::config::{BIG_STRIDE, TRAMP_CONTEXT};
use crate::loader::get_app_data;
use crate::mm::address::{PhysPageNum, VirtAddr};
use crate::mm::memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
use crate::my_struct::MyRefCell::MyRefCell;
use crate::task::context::TaskContext;
use crate::task::pid::{KernelStack, PidHandle};
use crate::trap::context::TrapFrame;
use crate::trap::trap_handler;
use crate::DEBUG;
use alloc::rc::Weak;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::RefMut;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    Uninit,  //未初始化
    Ready,   //准备执行
    Running, //正在执行
    Exited,  //已经退出
    Zombie,  //僵尸进程
}
pub struct TaskControlBlock {
    //不可变数据
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    //可变数据
    inner: MyRefCell<TaskControlBlockInner>,
}
pub struct TaskControlBlockInner {
    pub task_cx_ptr: TaskContext, //任务上下文栈顶地址的位置,位于内核空间中
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,                  //任务地址空间
    pub trap_cx_ppn: PhysPageNum,               //trap上下文所在的物理块
    pub base_size: usize,                       //应用程序的大小
    pub exit_code: usize,                       //保存退出码
    pub parent: Option<Weak<TaskControlBlock>>, //父进程
    pub children: Vec<Arc<TaskControlBlock>>,   //子进程需要引用计数

    pub stride: usize, //已走步长
    pub pass: usize,   //每一步的步长，只与特权级相关
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapFrame {
        //返回应用的trap上下文
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        //获取当前任务的用户地址空间根页表satp
        self.memory_set.token()
    }
    pub fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.task_status == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    pub fn new() -> Self {
        todo!("新建一个进程");
    }
    pub fn get_inner_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        //获取内部数据的可变借用
        self.inner.get_mut()
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }
    pub fn exec(&self, elf_data: &[u8]) {
        todo!(完成执行程序)
    }
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        //创建一个新的进程
        todo!()
    }
}
