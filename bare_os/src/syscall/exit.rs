use crate::batch::run_next_app;
use crate::println;

pub fn sys_exit(xstate:i32)->!{
    println!("[kernel] Application exited with code {}",xstate);
    //函数退出后，运行下一个应用程序

    run_next_app()
}