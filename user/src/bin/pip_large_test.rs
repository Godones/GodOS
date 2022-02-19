#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use lib::{close, fork, get_time_ms, pipe, println, read, wait, write};

const LENGTH: usize = 3000;
#[no_mangle]
pub fn main() -> i32 {
    // create pipes
    // parent write to child
    let mut down_pipe_fd = [0usize; 2];
    // child write to parent
    let mut up_pipe_fd = [0usize; 2];
    pipe(&mut down_pipe_fd);
    pipe(&mut up_pipe_fd);
    let mut random_str = [0u8; LENGTH];
    if fork() == 0 {
        // close write end of down pipe
        close(down_pipe_fd[1]); //关闭下载的写端
                                // close read end of up pipe
        close(up_pipe_fd[0]); //关闭上传的读端
        assert_eq!(read(down_pipe_fd[0], &mut random_str) as usize, LENGTH); //从父进程下载内容
        close(down_pipe_fd[0]);
        let sum: usize = random_str.iter().map(|v| *v as usize).sum::<usize>(); //从子进程上传内容给父进程
        println!("sum = {}(child)", sum);
        let sum_str = format!("{}", sum);
        write(up_pipe_fd[1], sum_str.as_bytes());
        close(up_pipe_fd[1]);
        println!("Child process exited!");
        0
    } else {
        // close read end of down pipe
        close(down_pipe_fd[0]);
        // close write end of up pipe
        close(up_pipe_fd[1]);
        // generate a long random string
        for i in 0..LENGTH {
            random_str[i] = get_time_ms() as u8;
        }
        // send it
        assert_eq!(
            write(down_pipe_fd[1], &random_str) as usize,
            random_str.len()
        );
        // println!("Father write over...");
        // close write end of down pipe
        close(down_pipe_fd[1]);
        // calculate sum(parent)
        let sum: usize = random_str.iter().map(|v| *v as usize).sum::<usize>();
        println!("sum = {}(parent)", sum);
        // recv sum(child)
        let mut child_result = [0u8; 32];
        let result_len = read(up_pipe_fd[0], &mut child_result) as usize;
        close(up_pipe_fd[0]);
        // check
        assert_eq!(
            sum,
            str::parse::<usize>(core::str::from_utf8(&child_result[..result_len]).unwrap())
                .unwrap()
        );
        let mut _unused: i32 = 0;
        wait(&mut _unused); //等待子进程结束
        println!("pipe_large_test passed!");
        0
    }
}
