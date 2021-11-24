use crate::sbi::*;
use core::fmt::{self, Arguments, Write};

struct Stdio;

impl Write  for Stdio{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        /// 这里传入的是一个字符串，而sbi只能按照u8输出，因此需要坐转换
        let mut buffer = [0u8;4];
        for c in s.chars(){
            for u8code in c.encode_utf8(& mut buffer).as_bytes().iter(){
                console_putchar(*u8code as usize);
            }
        }
        Ok(())
    }
}

pub fn print(args:fmt::Arguments){
    Stdio.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt:literal $(,$(arg:tt)+)?) => {
        $crate::console::print(format_args!($fmt $(,$($args)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}