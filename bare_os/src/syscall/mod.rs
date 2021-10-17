mod process;
use process::*;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
pub fn syscall(function: usize, args: [usize; 3]) -> isize {
    // crate::println!("function: {}, args: {:?}",function,args);
    match function {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        _ => {
            panic!("Undefined function for syscallfunction: {}", function);
        }
    }
}
