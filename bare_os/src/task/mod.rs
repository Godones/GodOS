use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx, run_next_app};
use core::cell::RefCell;
use lazy_static::lazy_static;
use switch::__switch;
use task::{TaskControlBlock,TaskStatus};

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
     static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock{task_cx_ptr:0,task_status:TaskStatus::Uninit};
            MAX_APP_NUM
        ];
        // todo!("注意这个位置");
        for i in 0..num_app{
            tasks[i].task_cx_ptr = init_app_cx(i) as *const usize as usize;
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
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].task_status = TaskStatus::Ready;
    }
    fn mark_current_exited(&self) {
        let current_task = self.inner.borrow().current_task;
        self.inner.borrow_mut().tasks[current_task].task_status = TaskStatus::Exited;
    }
    fn find_next_task(&self) -> Option<usize> {
        //寻找下一个可行的任务
        let inner = self.inner.borrow_mut();
        let current_task = inner.current_task;
        (current_task+1..current_task + self.num_app + 1)
            .map(|x| x % self.num_app)
            .find(|index| {
                //找到处于准备状态的任务
                inner.tasks[*index].task_status == TaskStatus::Ready
            })
    }
    fn run_next_task(&self) {
        if let Some(index) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current_task = inner.current_task;
            inner.current_task = index;
            inner.tasks[index].task_status = TaskStatus::Running;
            let current_task_cx_ptr2 = inner.tasks[current_task].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[index].get_task_cx_ptr2();
            unsafe {
                __switch(current_task_cx_ptr2, next_task_cx_ptr2);
            }
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

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}
