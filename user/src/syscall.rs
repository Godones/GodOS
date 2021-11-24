const SYSCALL_EXIT: usize = 93;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_TIME: usize = 169;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_FORK: usize = 220;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_READ: usize = 63;

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
pub fn sys_exit(state: i32) -> isize {
    syscall(SYSCALL_EXIT, [state as usize, 0, 0]) //执行退出
}

/// 功能：负责暂停当前进程，用于进程调度
///
pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

/// 功能：负责获取当前时间
pub fn sys_get_time() -> isize {
    syscall(SYSCALL_TIME, [0, 0, 0])
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
    syscall(SYSCALL_FORK,[0,0,0])
}

/// 功能：用于子进程的回收工作
/// 通过收集子进程的返回状态，决定是否回收相关的资源
/// `pid`表示子进程的pid,exit_code保存子进程返回值的地址
/// 子进程不存在返回-1，所有子进程均未结束返回-2,成功返回子进程的pid
/// syscall id 260
pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID,[pid as usize,exit_code as usize,0])
}

/// 功能：清空当前进程的内容并将新的应用程序加载到地址空间中
/// 返回用户态开始执行此进程
/// syscall id 221
pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
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
