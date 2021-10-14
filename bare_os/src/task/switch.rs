global_asm!(include_str!("switch.asm"));

extern "C" {
    fn _switch(current_task_ptr2: *const usize, next_task_ptr2: *const usize);
}
