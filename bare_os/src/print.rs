use core::fmt;
use core::fmt::Write;
const SYSCALL_EXIT:usize = 93;
const SYSCALL_WRITE:usize = 64;

#[warn(deprecated)]
fn syscall(id:usize,args:[usize;3])->isize{
    let mut ret: isize = 0;
    unsafe{//汇编指令
    llvm_asm!(
            "ecall"//调用中断指令
            :"={x10}"(ret) //输出操作数，只写操作数，位于x10寄存器
            :"{x10}"(args[0]),"{x11}"(args[1]),"{x12}"(args[2]),"{x17}"(id)
            //输入操作数
            :"memory" //代码将会改变内存的内容
            :"volatile" //禁止编译器对汇编程序进行优化
        );
        ret
    }
}
pub fn sys_exit(state:i32) ->isize{
    syscall(SYSCALL_EXIT,[state as usize,0,0])//执行退出
}

pub fn sys_write(fd:usize,buffer:&[u8])->isize{
    syscall(SYSCALL_WRITE,[fd,buffer.as_ptr() as usize,buffer.len()])
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

#[macro_export]
macro_rules! print {
    ($fmt:literal $(,$(arg:tt)+)?) => {
        $crate::print::print(format_args!($fmt $(,$($args)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

