use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::mm::address::VirtAddr;
use crate::mm::MapPermission;
use crate::mm::KERNEL_SPACE;
use crate::println;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

///完成进程描述符的创建于分配
trait PidAlloc {
    fn new() -> Self;
    fn alloc(&mut self) -> PidHandle;
    fn dealloc(&mut self, ppn: usize);
}
struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

pub struct PidHandle(pub usize);

impl PidAlloc for PidAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> PidHandle {
        if let Some(val) = self.recycled.pop() {
            PidHandle(val)
        } else {
            self.current += 1;
            PidHandle(self.current - 1)
        }
    }
    fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current); //判断是否已经分配出去过
        assert!(
            self.recycled.iter().find(|&ppid| *ppid == pid).is_none(),
            "pid {} has been dealloced",
            pid
        ); //是否已经回收过
        self.recycled.push(pid);
    }
}

lazy_static! {
    static ref PIDALLOCATOR: Mutex<PidAllocator> =  Mutex::new(PidAllocator::new());
}


pub fn pid_alloc() -> PidHandle {
    PIDALLOCATOR.lock().alloc()
}
impl Drop for PidHandle {
    fn drop(&mut self) {
        PIDALLOCATOR.lock().dealloc(self.0);
    }
}
//返回应用程序在内核的内核栈位置
fn kernel_stack_position(pid: usize) -> (usize, usize) {
    let top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let button = top - KERNEL_STACK_SIZE;
    (button, top)
}

pub struct KernelStack {
    //应用的内核栈
    pid: usize,
}

impl KernelStack {
    pub fn new(pidhandle: &PidHandle) -> Self {
        //为进程分配一个内核栈
        let (stack_button, stack_top) = kernel_stack_position(pidhandle.0);
        //直接插入应用的内核栈位置,以framed形式
        KERNEL_SPACE.lock().insert_framed_area(
            stack_button.into(),
            stack_top.into(),
            MapPermission::W | MapPermission::R,
        );
        Self { pid: pidhandle.0 }
    }
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
        let (_, top) = kernel_stack_position(self.pid);
        top
    }
    pub fn get_stack_button(&self) -> usize {
        let (button, _) = kernel_stack_position(self.pid);
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
