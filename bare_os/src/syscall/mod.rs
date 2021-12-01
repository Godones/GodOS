mod file;
mod process;

use crate::syscall::file::{sys_read, sys_write};
use process::*;

use crate::timer::Time;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_FORK: usize = 220;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_READ: usize = 63;
const SYSCALL_SPAWN: usize = 400;
const SYSCALL_PID: usize = 172;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_CLOSE: usize = 57;

pub fn syscall(call: usize, args: [usize; 3]) -> isize {
    // crate::println!("function: {}, args: {:?}",function,args);
    match call {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut Time),
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0]),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_SPAWN => sys_spawn(args[0] as *const u8),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_PID => sys_getpid(),
        SYSCALL_PIPE => sys_pipe(args[0] as usize as *mut usize),
        SYSCALL_CLOSE => sys_close(args[0]),
        _ => {
            panic!("Undefined call for syscall: {}", call);
        }
    }
}
