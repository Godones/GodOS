use crate::task::context::TaskContext;
use core::arch::global_asm;
global_asm!(include_str!("switch.asm"));

extern "C" {
    pub fn __switch(current_task_ptr2: *mut TaskContext, next_task_ptr2: *const TaskContext);
}
