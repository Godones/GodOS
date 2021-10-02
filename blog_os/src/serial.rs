///！目标：由于在我们测试时qemu窗口会一闪而过，无法查看相关的信息
///！因此我们需要想办法在宿主控制台打印相关的信息
///! 一个简单的方式是通过串行端口发送数据
/// todo!(学习一下串行端口的内容)
///! 这里我们选用uart_16650 crate来实现功能

use spin::Mutex;
use lazy_static::lazy_static;
use uart_16550::SerialPort;

lazy_static!{
    pub static ref SERIAL1:Mutex<SerialPort> = {
        let mut serialPort = unsafe{ SerialPort::new(0x3F8)};
        serialPort.init();
        Mutex::new(serialPort)
    };
}
use core::fmt::{Arguments, Write};
#[doc(hidden)]
pub fn _print(args:Arguments){
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed!");
}
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}
#[macro_export]
macro_rules! serial_println {
    (()) => {$crate::serial_print!(concat!($fmt,"\n"))};
    ($fmt:expr) => {$crate::serial_print!(concat!($fmt,"\n"))};
    ($fmt:expr,$($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt,"\n"),
        $($arg)*)
    }
}