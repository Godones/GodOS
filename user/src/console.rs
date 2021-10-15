use core::fmt::{self,Write};
use crate::write;

const STDOUT: usize = 1;


struct Stdout;
impl Write for Stdout{
    fn write_str(&mut self,s:&str)->fmt::Result{
        write(STDOUT,s.as_bytes());
        Ok(())
    }
}
pub fn print(args:fmt::Arguments){
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

