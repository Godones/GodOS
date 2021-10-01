use core::fmt;
use core::fmt::Write;
use volatile::Volatile;


///
/// 使用 #[allow(dead_code)]，我们可以对 Color 枚举类型禁用未使用变量警告。
/// repr(u8) 注记标注的枚举类型，都会以一个 u8 的形式存储
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// 存储前景色和背景色
// repr(transparent) 标记使得Colorcode和u8有相同的内存布局
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

//缓冲区
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
    //易失操作
    //为了告诉编译器我们在操作vga缓冲区，以避免编译后我们的修改操作被优化掉
    //需要使用valatile crate 对我们的数据进行包装
}

//打印字符
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            },//换行
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; //行
                let col = self.column_position;//列

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20...0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节,对应符号 ■
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn new_line(&mut self) {

        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;//恢复到开始的位置
    }
    fn clear_row(&mut self, row: usize) {
        let character = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(character);
        }
    }
}

impl fmt::Write for Writer {
    //提供格式化输出宏 write! /writeln!，需要实现 实现trait fmt::Write
    //只需要实现write_str即可
    fn write_str(&mut self, s: &str) -> fmt::Result {
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
    pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer{
    column_position :0,
    color_code : ColorCode::new(Color::Green,Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

///下面借用标准库的print!实现
/// $crate 变量使得我们不必在使用println!时导入宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]//防止在文档中生成
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}


pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        //危险操作,裸指针等使用
    };
    writer.write_byte(b'H');
    writer.write_string("ello ");
    write!(writer, "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();
}