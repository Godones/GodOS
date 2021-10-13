use crate::print;
const FUNCTION_STDOUT: usize = 1;
pub fn sys_write(function: usize, buf: *const u8, len: usize) -> isize {
    match function {
        FUNCTION_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            //未定义的操作
            panic!("Undefined function in sys_write");
        }
    }
}
