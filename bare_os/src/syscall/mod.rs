mod exit;
mod write;

use write::sys_write;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
use crate::syscall::exit::sys_exit;

pub fn syscall(function: usize, args: [usize; 3]) -> isize {
    // crate::println!("function: {}, args: {:?}",function,args);
    match function {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => {
            panic!("Undefined function for syscallfunction: {}", function);
        }
    }
}
