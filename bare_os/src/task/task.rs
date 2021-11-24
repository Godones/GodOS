use crate::config::{BIG_STRIDE, TRAMP_CONTEXT};
use crate::loader::get_data_by_name;
use crate::mm::address::{PhysPageNum, VirtAddr};
use crate::mm::memory_set::MemorySet;
use crate::mm::KERNEL_SPACE;
use crate::my_struct::my_ref_cell::MyRefCell;
use crate::task::context::TaskContext;
use crate::task::pid::{pid_alloc, KernelStack, PidHandle};
use crate::trap::context::TrapFrame;
use crate::trap::trap_handler;
use alloc::sync::{Arc,Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
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
    pub task_cx_ptr: TaskContext,               //任务上下文栈顶地址的位置,位于内核空间中
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,                  //任务地址空间
    pub trap_cx_ppn: PhysPageNum,               //trap上下文所在的物理块
    pub base_size: usize,                       //应用程序的大小
    pub exit_code: isize,                       //保存退出码
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
        //获取任务状态
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        //查看是否是僵尸进程
        self.task_status == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    pub fn new(task_name: &str) -> Self {
        let data = get_data_by_name(task_name).unwrap();
        //构造用户地址空间
        let (memory_set, use_sp, entry_point) = MemorySet::from_elf(data);
        //trap上下文所在物理页帧
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAMP_CONTEXT).into())
            .unwrap()
            .ppn(); //找到任务上下文对应的页表项并获得对应的物理页号

        //为进程分配pid
        let pid = pid_alloc();
        //根据pid为进程分配内核栈
        let kernel_stack = KernelStack::new(&pid);
        //获取内核栈顶
        let kernel_stack_top = kernel_stack.get_stack_top();
        let task_control_block = Self {
            pid,
            kernel_stack,
            inner: MyRefCell::new(TaskControlBlockInner {
                task_status: TaskStatus::Ready,
                task_cx_ptr: TaskContext::goto_trap_return(kernel_stack_top),
                memory_set,
                trap_cx_ppn,
                base_size: use_sp,
                exit_code: 0,
                parent: None,
                children: Vec::new(),
                stride: 0,
                pass: BIG_STRIDE / 2,
            }),
        }; //构造任务控制块

        let trap_cx = task_control_block.get_inner_access().get_trap_cx();

        *trap_cx = TrapFrame::app_into_context(
            entry_point,
            use_sp,
            KERNEL_SPACE.lock().token(), //内核地址空间的根页表
            kernel_stack_top,
            trap_handler as usize,
        ); //构造trap上下文写入内存中
        task_control_block
    }
    pub fn get_inner_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        //获取内部数据的可变借用
        self.inner.get_mut()
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }
    pub fn exec(&self, elf_data: &[u8]) {
        //更换当前进程的数据
        let (memoryset,user_sp,entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memoryset
            .translate(VirtAddr::from(TRAMP_CONTEXT).into())
            .unwrap()
            .ppn();
        let mut inner = self.get_inner_access();
        //更换地址空间和
        inner.memory_set = memoryset;
        inner.trap_cx_ppn = trap_cx_ppn;
        inner.base_size = user_sp;

        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapFrame::app_into_context(
            entry_point,//新的入口
            user_sp,//新的用户栈
            KERNEL_SPACE.lock().token(),
            self.kernel_stack.get_stack_top(),//原有的内核栈
            trap_handler as usize
        )
    }
    pub fn fork(self:&Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        //fork一个新的进程
        //构造用户地址空间
        let mut parent_inner = self.get_inner_access();
        //复制地址空间已经数据
        let memory_set = MemorySet::from_existed_memset(&parent_inner.memory_set);
        //trap上下文所在物理页帧
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAMP_CONTEXT).into())
            .unwrap()
            .ppn(); //找到任务上下文对应的页表项并获得对应的物理页号

        //为进程分配pid
        let pid = pid_alloc();
        //根据pid为进程分配内核栈
        let kernel_stack = KernelStack::new(&pid);
        //获取内核栈顶
        let kernel_stack_top = kernel_stack.get_stack_top();

        let task_control_block = Arc::new(Self{
            pid,
            kernel_stack,
            inner: MyRefCell::new(TaskControlBlockInner {
                task_status: TaskStatus::Ready,
                task_cx_ptr: TaskContext::goto_trap_return(kernel_stack_top),
                memory_set,
                trap_cx_ppn,
                base_size:parent_inner.base_size,
                exit_code: 0,
                parent: Some(Arc::downgrade(self)),//弱引用
                children:Vec::new(),
                stride: 0,
                pass: BIG_STRIDE / 2,
            }),
        }); //构造任务控制块
        parent_inner.children.push(task_control_block.clone());
        let trap_cx = task_control_block.get_inner_access().get_trap_cx();
         //构造trap上下文写入内存中
        trap_cx.kernel_sp = kernel_stack_top;
        task_control_block
        //todo!("为什么子进程的用户栈不需要更改")
    }

}
