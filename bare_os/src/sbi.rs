#![allow(dead_code)]
///使用RustSBI接口进行相关操作
///
const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

fn sbi_call(function: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        //汇编指令
        asm!(
            "ecall",//调用中断指令
            inlateout("x10") arg0 => ret, //输出操作数，只写操作数，位于x10寄存器
            in("x11") arg1,
            in("x12") arg2,
            in("x17") function,
            //输入操作数
            options(nostack)//代码将会改变内存的内容
            // :"volatile" //禁止编译器对汇编程序进行优化
        );
        ret
    }
}
//向控制台输出一个字符
fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}
//从控制台读取数据
fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    // panic!("It should shutdown\n");
    panic!()
}
pub fn set_timer(time: usize) {
    //设置mtimecmp的值
    sbi_call(SBI_SET_TIMER, time, 0, 0);
}

pub struct Console;
impl Console {
    pub fn write_byte(&mut self, byte: usize) {
        console_putchar(byte);
    }
    pub fn write_string(&mut self, s: &str) {
        for char in s.bytes() {
            self.write_byte(char as usize);
        }
    }
}

use core::fmt::{Arguments, Write};
impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

//延迟初始化，在第一次使用此变量时进行初始化。
//laze_static
//自旋锁防止数据竞争
//由于将writer声明为可变借用会导致写入错误
use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console);
}
#[doc(hidden)] //防止在文档中生成
pub fn _print(args: Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}
///借用标准库的print!实现
/// $crate 变量使得我们不必在使用println!时导入宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sbi::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

///彩色输出，用于不同信息之间的分隔
///表示发生严重错误，很可能或者已经导致程序崩溃
///
#[macro_export]
macro_rules! ERROR {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        ($crate::print!("\x1b[31m{}\x1b[0m\n", format_args!($($arg)*)));
    }
}

// 表示发生不常见情况，但是并不一定导致系统错误

#[macro_export]
macro_rules! WARN {
    () => (
        #[cfg(any(feature = " WARN ", feature = "INFO", feature = "DEBUG", feature = "TRACE"))]
        $crate::print!("\n");
        $crate::print!("");
    );
    ($($arg:tt)*) => {
        #[cfg(any(feature = " WARN ", feature = "INFO", feature = "DEBUG", feature = "TRACE"))]
        ($crate::print!("\x1b[93m{}\x1b[0m\n", format_args!($($arg)*)));
        $crate::print("");
    }
}

// 比较中庸的选项，输出比较重要的信息，比较常用

#[macro_export]
macro_rules! INFO {
    () => {
        #[cfg(any(feature = "INFO",feature = "DEBUG", feature = "TRACE"))]
        $crate::print!("\n");
        $crate::print!("");
    };
    ($($arg:tt)*) => {
        #[cfg(any(feature = "INFO",feature = "DEBUG", feature = "TRACE"))]
        ($crate::print!("\x1b[34m{}\x1b[0m\n", format_args!($($arg)*)));
        $crate::print!("");
    }
}

#[macro_export]
macro_rules! DEBUG {
    () => {
        #[cfg(any(feature = "DEBUG", feature = "TRACE"))]
        $crate::print!("\n")
        $crate::print!("");
    };
    ($($arg:tt)*) => {
        #[cfg(any(feature = "DEBUG", feature = "TRACE"))]
        ($crate::print!("\x1b[32m{}\x1b[0m\n", format_args!($($arg)*)));
        $crate::print!("");
    }
}

// 最详细的输出，跟踪了每一步关键路径的执行

#[macro_export]
macro_rules! TRACE {
    () => {
        #[cfg(feature = "TRACE")]
        $crate::print!("\n")
        $crate::print!("");
    };
    ($($arg:tt)*) => {
        #[cfg(feature = "TRACE")]
        ($crate::print!("\x1b[90m{}\x1b[0m\n", format_args!($($arg)*)));
        $crate::print!("");
    }
}
