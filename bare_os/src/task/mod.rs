use crate::config::{MAX_APP_NUM,BIG_STRIDE};
use crate::loader::{get_num_app, init_app_cx};
use core::cell::RefCell;
use lazy_static::lazy_static;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};


/// 为了更好地完成任务上下文切换，需要对任务处于什么状态做明确划分
///任务的运行状态：未初始化->准备执行->正在执行->已退出
pub mod context;
mod switch;
mod task;

//管理各个任务的任务管理器
pub struct TaskManager {
    num_app: usize,
    //应用数量
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    current_task: usize,
    //当前任务
    tasks: [TaskControlBlock; MAX_APP_NUM],
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    /// 初始化任务管理器
    /// 将各个应用的内核初始化完成 --- init_app_cx
    /// 将各个任务的状态改变为初始化完成状态
     static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock{
                task_cx_ptr: 0,
                task_status: TaskStatus::Uninit,
                pass: BIG_STRIDE/16,
                stride: 0,
            };
            MAX_APP_NUM
        ];
        // todo!("注意这个位置");
        for i in 0..num_app{
            // init_app_cx会返回一个任务上下文，task_cx_ptr保存的是任务上下文的引用的指针
            tasks[i].task_cx_ptr = init_app_cx(i) as *const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
        }
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
    fn set_priority(&self,priority:usize)->isize{
        //设置优先级就等价于更改增长量
        let  mut inner = self.inner.borrow_mut();
        let current_task = inner.current_task;
        inner.tasks[current_task].pass = BIG_STRIDE/priority;

        priority as isize
    }
    fn rr(&self) ->Option<usize>{
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

    fn stride(&self) ->Option<usize>{
        //stride调度算法
        let mut miniest =usize::MAX ;
        let mut index  = 0;
        for i in 0..self.num_app{
            if self.inner.borrow().tasks[i].stride < miniest&&
                self.inner.borrow().tasks[i].task_status==TaskStatus::Ready{
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
        let mut inner = self.inner.borrow_mut();
        inner.tasks[0].task_status = TaskStatus::Running;
        inner.tasks[0].stride += inner.tasks[0].pass;
        let next_task_ptr2 = inner.tasks[0].get_task_cx_ptr2();
        let _unused: usize = 0;
        drop(inner);
        unsafe {
            __switch(&_unused as *const usize, next_task_ptr2);
        }
    }
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            //查询是否有处于准备的任务，如果有就运行
            //否则退出
            let mut inner = self.inner.borrow_mut();
            let current_task = inner.current_task;
            inner.current_task = next;
            inner.tasks[next].task_status = TaskStatus::Running;

            inner.tasks[next].stride +=inner.tasks[next].pass;
            //获取两个任务的task上下文指针
            let current_task_cx_ptr2 = inner.tasks[current_task].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();

            //释放可变借用，否则进入下一个任务后将不能获取到inner的使用权
            core::mem::drop(inner);
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

pub fn set_priority(priority:usize) ->isize{
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
