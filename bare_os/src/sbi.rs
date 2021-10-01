#![feature(llvm_asm)]
///使用RustSBI接口进行相关操作
///
const SBI_SET_TIMER:usize = 0;
const SBI_CONSOLE_PUTCHAR :usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI:usize = 3;
const SBI_SEND_IPI:usize = 4;
const SBI_REMOTE_FENCE_I :usize = 5;
const SBI_REMOTE_SFENCE_VMA:usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID:usize =7;
const SBI_SHUTDOWN:usize = 8;

fn sbi_call(function:usize,arg0: usize,arg1:usize,arg2:usize)->usize{
    let mut ret = 0;
    unsafe {
        //汇编指令
        llvm_asm!(
            "ecall"//调用中断指令
            :"={x10}"(ret) //输出操作数，只写操作数，位于x10寄存器
            :"{x10}"(arg0),"{x11}"(arg1),"{x12}"(arg2),"{x17}"(function)
            //输入操作数
            :"memory" //代码将会改变内存的内容
            :"volatile" //禁止编译器对汇编程序进行优化
        );
        ret
    }
}
//向控制台输出一个字符
pub fn console_putchar(c:usize){
    sbi_call(SBI_CONSOLE_PUTCHAR,c ,0,0);
}
//从控制台读取数据
pub fn console_getchar()->usize{
    sbi_call(SBI_CONSOLE_GETCHAR,0,0,0)
}
fn shutdown() ->!{
    sbi_call(SBI_SHUTDOWN,0,0,0);
    panic!("It should shutdown\n");
}