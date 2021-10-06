use core::panic::PanicInfo;
use core::fmt::{self,Write};
const SYSCALL_EXIT:usize = 93;
const SYSCALL_WRITE:usize = 64;

/// 功能：将内存中缓冲区中的数据写入文件。
/// 参数：`fd` 表示待写入文件的文件描述符；
///      `buf` 表示内存中缓冲区的起始地址；
///      `len` 表示内存中缓冲区的长度。
/// 返回值：返回成功写入的长度。
/// syscall ID：64

pub fn sys_write(fd:usize,buffer:&[u8])->isize{
    syscall(SYSCALL_WRITE,[fd,buffer.as_ptr() as usize,buffer.len()])
}

/// 功能：退出应用程序并将返回值告知批处理系统。
/// 参数：`xstate` 表示应用程序的返回值。
/// 返回值：该系统调用不应该返回。
/// syscall ID：93
pub fn sys_exit(state:i32) ->isize{
    syscall(SYSCALL_EXIT,[state as usize,0,0])//执行退出
}


struct Stdout;
impl Write for Stdout{
    fn write_str(&mut self,s:&str)->fmt::Result{
        sys_write(1,s.as_bytes());
        Ok(())
    }
}

pub fn print(args:fmt::Arguments){
    Stdout.write_fmt(args).unwrap();
}
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize =-1;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}