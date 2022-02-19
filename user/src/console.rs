use crate::{read, write};
use core::fmt::{self, Write};

const STDOUT: usize = 1;
const STDIN: usize = 0;
struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

pub fn getchar() -> u8 {
    let mut buf = [0u8; 1];
    read(STDIN, &mut buf);
    buf[0]
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

#[macro_export]
macro_rules! INFO {
    () => {
        $crate::print!("");
    };
    ($($arg:tt)*) => {
        ($crate::print!("\x1b[34m{}\x1b[0m", format_args!($($arg)*)));
    }
}
