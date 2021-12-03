#![no_main]
#![no_std]

extern crate alloc;
use alloc::string::String;
use lib::console::getchar;
use lib::{exec, fork, print, println, wait_pid};

const LF: u8 = 10; //换行键
const CR: u8 = 13; //回车键
const DEL: u8 = 127; //删除键
const BS: u8 = 8; //退格键

#[no_mangle]
fn main() -> isize {
    println!("The User Shell");
    let mut process_name = String::new();
    print!("GodOS#");
    loop {
        let ch = getchar();
        match ch {
            LF | CR => {
                //回车或换行时
                println!(""); //换行
                if !process_name.is_empty() {
                    process_name.push('\0');
                    let pid = fork();
                    if pid == 0 {
                        //子进程
                        let info = exec(process_name.as_str());
                        if info == -1 {
                            //执行失败
                            println!("The error occurs when executing");
                            return -4;
                        }
                    } else {
                        //父进程
                        let mut exit_code: i32 = 0;
                        //等待子进程完成
                        let exit_pid = wait_pid(pid as usize, &mut exit_code);
                        if exit_pid == pid {
                            println!("Shell: Process {} exited with code {}", pid, exit_code);
                        }
                    }
                    process_name.clear();
                }
                print!("GodOS#");
            }
            DEL | BS => {
                //退格键
                if !process_name.is_empty() {
                    process_name.pop(); //删除最后一个字符
                    print!("{}", BS as char); //移动光标往前一个字符
                    print!(" ");
                    print!("{}", BS as char);
                }
            }
            _ => {
                process_name.push(ch as char);
                print!("{}", ch as char); //打印在屏幕上
            }
        }
    }
}
