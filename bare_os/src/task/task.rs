use crate::config::{kernel_stack_position, BIG_STRIDE, TRAMP_CONTEXT};
use crate::loader::get_app_data;
use crate::mm::address::{PhysPageNum, VirtAddr};
use crate::mm::memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
use crate::task::context::TaskContext;
use crate::trap::context::TrapFrame;
use crate::trap::trap_handler;
use crate::DEBUG;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TaskStatus {
    Uninit,  //未初始化
    Ready,   //准备执行
    Running, //正在执行
    Exited,  //已经退出
}

pub struct TaskControlBlock {
    pub task_cx_ptr: TaskContext, //任务上下文栈顶地址的位置,位于内核空间中
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,    //任务地址空间
    pub trap_cx_ppn: PhysPageNum, //trap上下文所在的物理块
    pub base_size: usize,         //应用程序的大小

    pub stride: usize, //已走步长
    pub pass: usize,   //每一步的步长，只与特权级相关
}

impl TaskControlBlock {
    //返回指向task栈顶的指针 -> task_cx_ptr 里面存的是task上下文的地址的地址
    // pub fn get_task_cx_ptr2(&self) -> *const usize {
    //     &self.task_cx_ptr as *const usize //指向指针的指针
    // }
}

impl TaskControlBlock {
    //创建任务控制块
    pub fn new(app_id: usize) -> Self {
        let data = get_app_data(app_id);
        //构造用户地址空间
        let (memory_set, use_sp, entry_point) = MemorySet::from_elf(data);
        //trap上下文所在物理页帧
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAMP_CONTEXT).into())
            .unwrap()
            .ppn(); //找到任务上下文对应的页表项并获得对应的物理页号
        let task_status = TaskStatus::Ready; //准备状态
                                             //映射用户在内核空间的栈空间

        let (button, top) = kernel_stack_position(app_id);
        //直接插入应用的内核栈位置,以framed形式
        KERNEL_SPACE.lock()
            .insert_framed_area(
            button.into(),
            top.into(),
            MapPermission::W | MapPermission::R,
        );

        //应用内核栈顶位置,我们需要放置一个任务上下文来切换到trap处理段
        // let task_cx_ptr = (top - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
        DEBUG!(
            "[kernel] {} app",app_id);
        // unsafe {
        //     *task_cx_ptr = TaskContext::goto_trap_return();
        // }
        let task_control_block = TaskControlBlock {
            task_status,
            task_cx_ptr: TaskContext::goto_trap_return(top),
            memory_set,
            trap_cx_ppn,
            base_size: use_sp, //在应用地址空间的栈顶位置就是整个应用的大小
            stride: 0,
            pass: BIG_STRIDE / 2,
        }; //构造任务控制块

        let trap_cx = task_control_block.get_trap_cx();

        *trap_cx = TrapFrame::app_into_context(
            entry_point,
            use_sp,
            KERNEL_SPACE.lock().token(), //内核地址空间的根页表
            top,
            trap_handler as usize,
        );
        //构造trap上下文写入内存中
        task_control_block
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapFrame {
        //返回应用的trap上下文
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        //获取当前任务的用户地址空间根页表satp
        self.memory_set.token()
    }
}
