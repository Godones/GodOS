#![no_std]
#![no_main]

#[macro_use]
extern crate lib;

use lib::{close, fork, pipe, read, wait, write};

static STR: &str = "Hello, world!";
// static STRs :&[u8]= &[55 as u8;1000];
#[no_mangle]
pub fn main() -> i32 {
    // create pipe
    let mut pipe_fd = [0usize; 2];
    pipe(&mut pipe_fd);
    // read end
    println!("pipefd {},{}", pipe_fd[0], pipe_fd[1]);
    assert_eq!(pipe_fd[0], 4);
    // write end
    assert_eq!(pipe_fd[1], 5);
    if fork() == 0 {
        // child process, read from parent
        // close write_end
        close(pipe_fd[1]);
        let mut buffer = [0u8; 1010];
        let len_read = read(pipe_fd[0], &mut buffer) as usize;
        // close read_end
        close(pipe_fd[0]);
        assert_eq!(core::str::from_utf8(&buffer[..len_read]).unwrap(), STR);
        assert_eq!(len_read, 13);
        println!("Read OK, child process exited!");
        0
    } else {
        // parent process, write to child
        // close read end
        close(pipe_fd[0]);
        let str = core::str::from_utf8(STR.as_ref()).unwrap();
        assert_eq!(write(pipe_fd[1], STR.as_bytes()), str.len() as isize);
        // close write end
        close(pipe_fd[1]);
        let mut child_exit_code: i32 = 0;
        wait(&mut child_exit_code);
        assert_eq!(child_exit_code, 0);
        println!("pipetest passed!");
        0
    }
}
