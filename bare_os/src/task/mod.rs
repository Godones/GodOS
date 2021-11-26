use crate::config::BIG_STRIDE;
use crate::loader::get_num_app;
use crate::task::context::TaskContext;
use crate::trap::context::TrapFrame;
use crate::INFO;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::iter::Map;
use lazy_static::lazy_static;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use crate::mm::address::VirtPageNum;
use crate::mm::memory_set::MapArea;

/// 为了更好地完成任务上下文切换，需要对任务处于什么状态做明确划分
///任务的运行状态：未初始化->准备执行->正在执行->已退出
pub mod context;
mod switch;
mod task;

pub static mut TASKLOADED: bool = false;
//管理各个任务的任务管理器
pub struct TaskManager {
    num_app: usize,
    //应用数量
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    //当前任务
    current_task: usize,
    tasks: Vec<TaskControlBlock>,
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    /// 初始化任务管理器
    /// 将各个应用的内核初始化完成 --- init_app_cx
    /// 将各个任务的状态改变为初始化完成状态
     static ref TASK_MANAGER: TaskManager = {
        INFO!("[kernel] init application...");
        let num_app = get_num_app();
        INFO!("[kernel] The app num: {}",num_app);

        let mut tasks :Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app{
            tasks.push(TaskControlBlock::new(i));
        }
        INFO!("[kernel] all application has loaded done");
        TaskManager{
            num_app,
            inner: RefCell::new(
                TaskManagerInner{
                    tasks,
                    current_task:0
            }
        )
    }
};
}
impl TaskManager {
    fn get_current_token(&self) -> usize {
        //获取用户地址空间的satp
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow().tasks[current_task].get_user_token()
    }
    fn get_trap_cx(&self) -> &'static mut TrapFrame {
        //获取用户trap上下文所在位置
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].get_trap_cx()
    }
    fn add_area(&self,mut area: MapArea){
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].memory_set.push(area,None);
    }

    fn mark_current_suspended(&self) {
        //将当前任务变成暂停状态
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        // 退出当前任务
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].task_status = TaskStatus::Exited;
    }
    fn set_priority(&self, priority: usize) -> isize {
        //设置优先级就等价于更改增长量
        let mut inner = self.inner.borrow_mut();
        let current_task = inner.current_task;
        inner.tasks[current_task].pass = BIG_STRIDE / priority;

        priority as isize
    }
    fn rr(&self) -> Option<usize> {
        //这个是简单的时间片轮转法
        let inner = self.inner.borrow_mut();
        let current_task = inner.current_task;
        (current_task + 1..current_task + self.num_app + 1)
            .map(|x| x % self.num_app)
            .find(|index| {
                //找到处于准备状态的任务
                inner.tasks[*index].task_status == TaskStatus::Ready
            })
    }

    fn stride(&self) -> Option<usize> {
        //stride调度算法
        let mut miniest = usize::MAX;
        let mut index = 0;
        for i in 0..self.num_app {
            if self.inner.borrow().tasks[i].stride < miniest
                && self.inner.borrow().tasks[i].task_status == TaskStatus::Ready
            {
                miniest = self.inner.borrow().tasks[i].stride;
                index = i;
            }
        }
        // DEBUG!("[kernel debug] {} {}",miniest,index);
        Some(index)
    }

    fn find_next_task(&self) -> Option<usize> {
        //寻找下一个可行的任务
        self.rr()
        // self.stride()
    }
    fn run_first_task(&self) {
        //需要在第一个任务运行前设置第一个时钟周期
        let mut inner = self.inner.borrow_mut();
        inner.tasks[0].task_status = TaskStatus::Running;
        inner.tasks[0].stride += inner.tasks[0].pass;
        //第一个应用程序的任务上下文
        let next_task_ptr2 = &inner.tasks[0].task_cx_ptr as *const TaskContext;

        let mut _unused = TaskContext::zero_init();
        drop(inner);
        INFO!(
            "[kernel] run the first application, address: {}",
            next_task_ptr2 as usize
        );

        unsafe {
            __switch(&mut _unused as *mut _, next_task_ptr2);
        }
    }
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            //查询是否有处于准备的任务，如果有就运行
            //否则退出
            let mut inner = self.inner.borrow_mut();
            let current_task = inner.current_task;
            // if next ==3{
            //     INFO!("[kernel] run the {} app, pre task {}", next,inner.current_task);
            // }
            inner.current_task = next;
            inner.tasks[next].task_status = TaskStatus::Running;

            inner.tasks[next].stride += inner.tasks[next].pass;
            //获取两个任务的task上下文指针
            let current_task_cx_ptr2 =
                &mut inner.tasks[current_task].task_cx_ptr as *mut TaskContext;
            let next_task_cx_ptr2 = &inner.tasks[next].task_cx_ptr as *const TaskContext;

            //释放可变借用，否则进入下一个任务后将不能获取到inner的使用权
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr2, next_task_cx_ptr2);
            }
        } else {
            panic!("There are no tasks!");
        }
    }
}

pub fn suspend_current_run_next() {
    mark_current_suspended(); //标记
    run_next_task(); //运行下一个
}

pub fn exit_current_run_next() {
    mark_current_exited();
    run_next_task();
}
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}
pub fn current_trap_cx() -> &'static mut TrapFrame {
    TASK_MANAGER.get_trap_cx()
}
pub fn current_add_area(mut area: MapArea)->isize{
    //添加一些页
    TASK_MANAGER.add_area(area);
    0
}
pub fn current_delete_page(page:VirtPageNum)->isize{
    TASK_MANAGER
    0
}


pub fn set_priority(priority: usize) -> isize {
    //设置特权级
    TASK_MANAGER.set_priority(priority)
}
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}
