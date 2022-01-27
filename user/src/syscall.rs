#![allow(unused)]
const SYSCALL_EXIT: usize = 93;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_TIME: usize = 169;
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
const SYSCALL_LS: usize = 44; //自定义系统调用
const SYSCALL_LINKAT:usize = 37;
const SYSCALL_UNLINKAT:usize = 35;
const SYSCALL_FSTAT:usize = 80;

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

use alloc::sync::Arc;
use crate::{Stat, Time};
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!("ecall",
        inlateout("x10") args[0] => ret,
        in("x11") args[1],
        in("x12") args[2],
        in("x17") id,
        options(nostack)
        )
    }
    ret
}
/// fn syscall6(id: usize, args: [usize; 6]) -> isize {
///     let mut ret: isize;
///     unsafe {
///         asm!("ecall",
///         inlateout("x10") args[0] => ret,
///         in("x11") args[1],
///         in("x12") args[2],
///         in("x13") args[3],
///         in("x14") args[4],
///         in("x15") args[5],
///         in("x17") id,
///         options(nostack)
///         )
///     }
///     ret
/// }
/// 功能：将内存中缓冲区中的数据写入文件。
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buf` 表示内存中缓冲区的起始地址；
///      `len` 表示内存中缓冲区的长度。
/// 返回值：返回成功写入的长度。
/// syscall ID：64
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`xstate` 表示应用程序的返回值。
/// 返回值：该系统调用不应该返回。
/// syscall ID：93
pub fn sys_exit(state: i32) -> ! {
    syscall(SYSCALL_EXIT, [state as usize, 0, 0]); //执行退出
    panic!("sys_exit never return");
}

/// 功能：负责暂停当前进程，用于进程调度
///
pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

/// 功能：负责获取当前时间
pub fn sys_get_time(time: &mut Time) -> isize {
    syscall(SYSCALL_TIME, [time as *mut Time as usize, 0, 0])
}
/// 功能：负责设置特权级
pub fn sys_set_priority(priority: isize) -> isize {
    syscall(SYSCALL_SET_PRIORITY, [priority as usize, 0, 0])
}
/// 功能：用于进程的创建
/// 对于父进程来说，其返回值是新创建的子进程的PID，对于子进程来说
/// 返回0
/// syscall id 220
pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

/// 功能：用于子进程的回收工作
/// 通过收集子进程的返回状态，决定是否回收相关的资源
/// `pid`表示子进程的pid,exit_code保存子进程返回值的地址
/// 子进程不存在返回-1，所有子进程均未结束返回-2,成功返回子进程的pid
/// syscall id 260
pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}

/// 功能：清空当前进程的内容并将新的应用程序加载到地址空间中
/// 返回用户态开始执行此进程
/// syscall id 221
pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
    syscall(
        SYSCALL_EXEC,
        [path.as_ptr() as usize, args.as_ptr() as usize, 0],
    )
}

/// 功能：从文件中或屏幕读取内容到缓冲区内
/// fd 是文件描述符，指向文件或者是屏幕，buffer未缓冲区
/// 出错时返回-1，否则返回读取的长度
pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(
        SYSCALL_READ,
        [fd, buffer.as_mut_ptr() as usize, buffer.len()],
    )
}

/// 功能：新建子进程并执行子进程
///
pub fn sys_spawn(path: &str) -> isize {
    syscall(SYSCALL_SPAWN, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_PID, [0, 0, 0])
}

///申请一个len长度的物理内存，将其映射到start开始的许村，内存页属性为port
///其中port等待0位表示是否可读，1位是否可写，2表示是否可执行
/// 成功返回0，错误返回-1
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    syscall(SYSCALL_MMAP, [start, len, port])
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    syscall(SYSCALL_MUNMAP, [start, len, 0])
}
pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0])
}
pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}

///读取邮箱的内容
pub fn sys_mail_read(buf: &mut [u8]) -> isize {
    syscall(SYSCALL_MAILREAD, [buf.as_mut_ptr() as usize, buf.len(), 0])
}
///向指定进程发送内容
pub fn sys_mail_write(pid: usize, buf: &mut [u8]) -> isize {
    syscall(
        SYSCALL_MAILWRITE,
        [pid, buf.as_mut_ptr() as usize, buf.len()],
    )
}

///功能：打开普通文件
///参数：`path`:文件名称
///flags:打开方式
///返回值：错误-1，成功返回文件描述符
pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}
///重定向
/// 功能：将进程中一个已经打开的文件复制一份并分配到一个新的文件描述符中。
/// 参数：fd 表示进程中一个已经打开的文件的文件描述符。
/// 返回值：如果出现了错误则返回 -1，否则能够访问已打开文件的新文件描述符。
/// 可能的错误原因是：传入的 fd 并不对应一个合法的已打开文件。
/// syscall ID：24
pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0])
}

/// ls
/// 功能：查看指定目录下的文件
/// 当前只能查看根目录下的文件
pub fn sys_ls() -> isize {
    syscall(SYSCALL_LS, [0, 0, 0])
}

/// 实现文件的硬连接
/// 这里只需要关注oldpath与newpath即可
///
pub fn sys_linkat(
    olddirfd: i32,
    oldpath: *const u8,
    newdirfd: i32,
    newpath: *const u8,
    flags: u32) -> isize{

    syscall(SYSCALL_LINKAT,[oldpath as usize,newpath as usize,0])
}
/// 解除一个文件的链接
pub fn sys_unlinkat(dirfd: i32, path: *const u8, flags: u32) -> isize{
    syscall(SYSCALL_UNLINKAT,[path as usize,0,0])
}
/// 查看文件信息
///
pub fn sys_fstat(fd:usize,stat:&Stat)->isize{
    syscall(SYSCALL_FSTAT,[fd,stat as *const Stat as usize,0])
}
/// 创建线程
/// entry:线程入口地址
/// arg:线程参数
pub fn sys_thread_create(entry:usize,arg:usize)->isize{
    syscall(SYSCALL_THREAD_CREATE,[entry,arg,0])
}
/// 回收线程资源
/// tid:线程标识符
pub fn sys_waittid(tid:usize)->isize{
    syscall(SYSCALL_WAITTID,[tid,0,0])
}

/// 创建互斥锁
pub fn sys_mutex_blocking_create()->isize{
    syscall(SYSCALL_MUTEX_CREATE,[1,0,0])
}

/// 申请加锁
pub fn sys_mutex_lock(lock_id:usize)->isize{
    syscall(SYSCALL_MUTEX_LOCK,[lock_id,0,0])
}

/// 申请释放锁
pub fn sys_mutex_unlock(lock_id:usize)->isize{
    syscall(SYSCALL_MUTEX_UNLOCK,[lock_id,0,0])
}

/// 创建自旋锁
pub fn sys_mutex_create()->isize{
    syscall(SYSCALL_MUTEX_CREATE,[0,0,0])
}

/// 创建信号量
pub fn sys_semaphore_create(count:usize)->isize{
    syscall(SYSCALL_SEMAPHORE_CREATE,[count,0,0])
}
/// 信号量p操作
pub fn sys_semaphore_p(sem_id:usize)->isize{
    syscall(SYSCALL_SEMAPHORE_DOWN,[sem_id,0,0])
}

/// 信号量v操作
pub fn sys_semaphore_v(sem_id:usize)->isize{
    syscall(SYSCALL_SEMAPHORE_UP,[sem_id,0,0])
}


pub fn sys_monitor_create()->isize{
    syscall(SYSCALL_MONITOR_CREATE,[0,0,0])
}

pub fn sys_monitor_wait(mon_id:usize,mutex_id:usize)->isize{
    syscall(SYSCALL_MONITOR_WAIT,[mon_id,mutex_id,0])
}
pub fn sys_monitor_signal(mon_id:usize)->isize{
    syscall(SYSCALL_MONITOR_SIGNAL,[0,0,0])
}