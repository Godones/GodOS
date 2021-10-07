mod write;
mod exit;

use write::sys_write;

const SYSCALL_WRITE :usize = 1;
const SYSCALL_EXIT: usize = 8;
use crate::syscall::exit::sys_exit;

pub fn syscall(function: usize, args:[usize;3]) -> isize{
    match function {
        SYSCALL_WRITE => {
            sys_write(args[0], args[1] as *const u8,args[2])
        }
        SYSCALL_EXIT => {
            sys_exit(args[0])
        }
        _ => {
            panic!("Undefined function for syscallfunction: {}",function);
        }
    }

 }
