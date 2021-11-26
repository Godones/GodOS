use crate::mm::page_table::{PageTable, translated_refmut};
use crate::task::set_priority;
use crate::task::suspend_current_run_next;
use crate::task::{current_user_token, exit_current_run_next};
use crate::timer::Time;
use crate::{print, INFO, println};
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
            let slice = PageTable::translated_byte_buffer(current_user_token(), buf, len);
            for buffer in slice {
                let str = core::str::from_utf8(buffer).unwrap();
                print!("{}", str);
            }
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
pub fn sys_get_time(time: *mut Time) -> isize {
    let current_time = crate::timer::get_cost_time(); //获取微秒
    // println!("current: {}",current_time);
    unsafe {
        *(translated_refmut(current_user_token(),time)) = Time {
            s: current_time / 1000_000,
            us: current_time % 1000_000,
        };
    }
    0
}
pub fn sys_set_priority(priority: usize) -> isize {
    //设置应用的特权级
    set_priority(priority)
}
