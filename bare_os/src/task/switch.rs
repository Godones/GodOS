use crate::task::context::TaskContext;
global_asm!(include_str!("switch.asm"));

extern "C" {
    pub fn __switch(current_task_ptr2: *const TaskContext, next_task_ptr2: *const TaskContext);
}
