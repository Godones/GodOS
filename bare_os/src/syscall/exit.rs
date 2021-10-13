use crate::loader::run_next_app;
use crate::INFO;

pub fn sys_exit(xstate: i32) -> ! {
    INFO!("[kernel] Application exited with code {}", xstate);
    //函数退出后，运行下一个应用程序

    run_next_app()
}
