use alloc::sync::{Arc, Weak};
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::mm::address::{PhysPageNum, VirtAddr};
use crate::mm::MapPermission;
use crate::mm::KERNEL_SPACE;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::DEBUG;
use crate::task::process::ProcessControlBlock;

pub struct RecycleAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl RecycleAllocator {
    pub fn new() -> Self {
        Self {
            current: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> usize {
        if let Some(val) = self.recycled.pop() {
            val
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.current); //判断是否已经分配出去过
        assert!(
            self.recycled.iter().find(|&i| *i == id).is_none(),
            "id {} has been dealloced",
            id
        ); //是否已经回收过
        self.recycled.push(id);
    }
}

lazy_static! {
    static ref PIDALLOCATOR: Mutex<RecycleAllocator> = Mutex::new(RecycleAllocator::new());
    static ref KERNEL_STACK_ALLOCATOR:Mutex<RecycleAllocator> = Mutex::new(RecycleAllocator::new());
}

pub struct PidHandle(pub usize);

pub fn pid_alloc() -> PidHandle {
    PidHandle(PIDALLOCATOR.lock().alloc())
}
impl Drop for PidHandle {
    fn drop(&mut self) {
        PIDALLOCATOR.lock().dealloc(self.0);
    }
}
//返回应用程序在内核的内核栈位置
fn kernel_stack_position(kernel_stack_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - kernel_stack_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let button = top - KERNEL_STACK_SIZE;
    (button, top)
}

pub struct KernelStack(usize);

pub fn kernel_stack_alloc()->KernelStack{
    let ks_id = KERNEL_STACK_ALLOCATOR.lock().alloc();
    let (button,top) = kernel_stack_position(ks_id);
    KERNEL_SPACE.lock().insert_framed_area(
        button.into(),
        top.into(),
        MapPermission::W | MapPermission::R,
    );
    KernelStack(ks_id)
}

impl KernelStack {
    pub fn push_data_top<T>(&self, val: T) -> *mut T
    where
        T: Sized,
    {
        let stack_top = self.get_stack_top();
        let start_ptr = (stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *start_ptr = val;
        }
        start_ptr
    }
    pub fn get_stack_top(&self) -> usize {
        let (_, top) = kernel_stack_position(self.0);
        top
    }
    pub fn get_stack_button(&self) -> usize {
        let (button, _) = kernel_stack_position(self.0);
        button
    }
}

impl Drop for KernelStack {
    //自动回收用户的内核栈
    //需要从地址空间中回收掉
    fn drop(&mut self) {
        let stack_button = self.get_stack_button();
        let button_viradd: VirtAddr = stack_button.into();
        KERNEL_SPACE.lock().remove_from_startaddr(button_viradd);
    }
}
///根据线程tid获取线程的trap上下文所在位置
fn trap_cx_button_from_tid(tid:usize)->usize{
    TRAP_CONTEXT - tid*PAGE_SIZE
}

/// 根据用户栈底和tid获取每个线程所在的用户栈的位置
fn ustack_bottom_from_tid(ustak_base:usize,tid:usize)->usize{
    ustak_base + tid * (PAGE_SIZE+USER_STACK_SIZE)
}
pub struct TaskUserRes{
    pub tid:usize,//线程描述符
    pub ustack_base:usize,//线程栈顶地址--用户态
    pub process:Weak<ProcessControlBlock>,
}

impl TaskUserRes {
    pub fn new(
        ustack_base:usize,
        process:Arc<ProcessControlBlock>,
        alloc_user_res:bool
    )->Self{
        let tid = process.get_inner_access().alloc_tid();
        let task_user_res = Self{
            tid,
            ustack_base,
            process:Arc::downgrade(&process),
        };
        if alloc_user_res {
            task_user_res.alloc_user_res();
        }
        task_user_res
    }
    pub fn alloc_user_res(&self){
        let process = self.process.upgrade().unwrap();
        let mut inner = process.get_inner_access();
        //声请线程用户栈
        let ustack_buttom = ustack_bottom_from_tid(self.ustack_base,self.tid);
        let ustack_top = ustack_buttom + USER_STACK_SIZE;
        inner.memory_set
            .insert_framed_area(
                ustack_buttom.into(),
                ustack_top.into(),
                MapPermission::R|MapPermission::W|MapPermission::U,
            );//插入地址空间中
        //获取trap上下文
        let trap_cx_bottom = trap_cx_button_from_tid(self.tid);
        let trap_cx_top = trap_cx_bottom + PAGE_SIZE;//
        inner.memory_set
            .insert_framed_area(
                trap_cx_bottom.into(),
                trap_cx_top.into(),
                MapPermission::R|MapPermission::W
            );//插入trap上下文
    }
    pub fn dealloc_user_res(&self){
        //回收线程所用的东西
        let process = self.process.upgrade().unwrap();
        let mut inner = process.get_inner_access();
        let ustack_bottom_va :VirtAddr= ustack_bottom_from_tid(self.ustack_base,self.tid).into();
        inner.memory_set.remove_from_startaddr(ustack_bottom_va);

        let trap_cx_bottom_va:VirtAddr = trap_cx_button_from_tid(self.tid).into();
        inner.memory_set.remove_from_startaddr(trap_cx_bottom_va);
    }
    pub fn dealloc_tid(&self){
        //回收线程描述符
        let process = self.process.upgrade().unwrap();
        let mut inner = process.get_inner_access();
        inner.dealloc_tid(self.tid)
    }
    //返回trap上下文
    pub fn trap_cx_user_va(&self)->usize{
        trap_cx_button_from_tid(self.tid)
    }

    pub fn trap_cx_ppn(&self)->PhysPageNum{
        let process = self.process.upgrade().unwrap();
        let inner = process.get_inner_access();
        let trap_cx_bottom_va:VirtAddr = trap_cx_button_from_tid(self.tid).into();
        inner.memory_set.translate(trap_cx_bottom_va.into()).unwrap().ppn()
    }
    pub fn ustack_top(&self)->usize{
        ustack_bottom_from_tid(self.ustack_base,self.tid) + USER_STACK_SIZE
    }

}

impl Drop for TaskUserRes {
    fn drop(&mut self) {
        self.dealloc_user_res();
        self.dealloc_tid();
    }
}