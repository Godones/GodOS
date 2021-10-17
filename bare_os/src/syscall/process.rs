use crate::task::exit_current_run_next;
use crate::task::suspend_current_run_next;
use crate::{print, INFO};

const FUNCTION_STDOUT: usize = 1;
pub fn sys_exit(xstate: i32) -> ! {
    INFO!("[kernel] Application exited with code {}", xstate);
    //函数退出后，运行下一个应用程序
    exit_current_run_next();
    panic!("Unreachable sys_exit!")
}

pub fn sys_write(function: usize, buf: *const u8, len: usize) -> isize {
    match function {
        FUNCTION_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            //未定义的操作
            panic!("Undefined function in sys_write");
        }
    }
}
pub fn sys_yield() -> isize {
    suspend_current_run_next(); //暂停当前任务运行下一个任务
    0
}
pub fn sys_get_time() -> isize {
    crate::timer::get_costtime() as isize
}
