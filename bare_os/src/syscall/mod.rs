mod file;
mod multhread;
mod process;
mod sync;

use crate::file::Stat;
use crate::syscall::file::*;
use crate::timer::Time;
use multhread::*;
use process::*;
use sync::*;

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
const SYSCALL_MAILREAD: usize = 401;
const SYSCALL_MAILWRITE: usize = 402;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_DUP: usize = 24;
const SYSCALL_LS: usize = 44; //自定义ls
const SYSCALL_LINKAT: usize = 37;
const SYSCALL_UNLINKAT: usize = 35;
const SYSCALL_FSTAT: usize = 80;

const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;

const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;
const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
const SYSCALL_SEMAPHORE_UP: usize = 1021;
const SYSCALL_SEMAPHORE_DOWN: usize = 1022;
const SYSCALL_MONITOR_CREATE: usize = 1030;
const SYSCALL_MONITOR_SIGNAL: usize = 1031;
const SYSCALL_MONITOR_WAIT: usize = 1032;

pub fn syscall(call: usize, args: [usize; 3]) -> isize {
    match call {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut Time),
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0]),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_SPAWN => sys_spawn(args[0] as *const u8),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_PID => sys_getpid(),
        SYSCALL_PIPE => sys_pipe(args[0] as usize as *mut usize),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_MAILREAD => sys_mail_read(args[0] as *mut u8, args[1]),
        SYSCALL_MAILWRITE => sys_mail_write(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_LS => sys_ls(),
        SYSCALL_FSTAT => sys_fstat(args[0], args[1] as *mut Stat),
        SYSCALL_LINKAT => sys_linkat(args[0] as *const u8, args[1] as *const u8),
        SYSCALL_UNLINKAT => sys_unlinkat(args[0] as *const u8),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEMAPHORE_CREATE => sys_semaphore_create(args[0]),
        SYSCALL_SEMAPHORE_DOWN => sys_semaphore_p(args[0]),
        SYSCALL_SEMAPHORE_UP => sys_semaphore_v(args[0]),
        SYSCALL_MONITOR_CREATE => sys_monitor_create(),
        SYSCALL_MONITOR_SIGNAL => sys_monitor_signal(args[0]),
        SYSCALL_MONITOR_WAIT => sys_monitor_wait(args[0], args[1]),
        _ => {
            panic!("Undefined call for syscall: {}", call);
        }
    }
}
