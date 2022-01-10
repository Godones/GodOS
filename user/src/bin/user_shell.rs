#![no_main]
#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use lib::console::getchar;
use lib::{close, dup, exec, fork, open, print, println, wait_pid, OpenFlags, INFO};

const LF: u8 = 10; //换行键
const CR: u8 = 13; //回车键
const DEL: u8 = 127; //删除键
const BS: u8 = 8; //退格键

fn command_parser(str: &str) -> (String, String, Vec<String>, Vec<*const u8>) {
    let command: Vec<_> = str.split_whitespace().collect();
    let mut command: Vec<String> = command
        .iter()
        .map(|&str| {
            let mut store_string = String::new();
            store_string.push_str(str); //转化为堆上的数据
            store_string
        })
        .collect();
    command.iter_mut().for_each(|str| {
        str.push('\0');
    }); //给每个字符串添加换行符
    let mut input = String::new();
    if let Some((idx, _)) = command
        .iter()
        .enumerate()
        .find(|(_, str)| str.as_str() == "<\0")
    {
        input = command[idx + 1].clone();
        command.drain(idx..=idx + 1);
    }
    let mut output = String::new();
    if let Some((idx, _)) = command
        .iter()
        .enumerate()
        .find(|(_, str)| str.as_str() == ">\0")
    {
        output = command[idx + 1].clone();
        command.drain(idx..=idx + 1);
    }

    let mut args: Vec<*const u8> = command.iter().map(|str| str.as_ptr()).collect(); //转为指针
    args.push(0 as *const u8);
    (input, output, command, args)
}

#[no_mangle]
fn main() -> isize {
    println!("The User Shell");
    let mut process_name = String::new();
    INFO!("GodOS:/\n$");
    loop {
        let ch = getchar();
        match ch {
            LF | CR => {
                //回车或换行时
                println!(""); //换行
                if !process_name.is_empty() {
                    let (input, output, command, args_addr) = command_parser(process_name.as_str());
                    let pid = fork();
                    if pid == 0 {
                        //子进程
                        //检查是否是否有重定向输入文件
                        if !input.is_empty() {
                            let input_fd = open(input.as_str(), OpenFlags::R);
                            if input_fd == -1 {
                                println!("can't open input file");
                                return -4; //打不开文件
                            }
                            let input_fd = input_fd as usize;
                            close(0); //将标准输入关闭
                            assert_eq!(dup(input_fd), 0); //检查是否将输入文件替换为标准输入文件
                            close(input_fd);
                        }
                        if !output.is_empty() {
                            let output_fd = open(output.as_str(), OpenFlags::W | OpenFlags::C);
                            if output_fd == -1 {
                                println!("can't open output file");
                                return -4; //打不开文件
                            }
                            let output_fd = output_fd as usize;
                            close(1); //将标准输入关闭
                            assert_eq!(dup(output_fd), 1); //检查是否将输入文件替换为标准输入文件
                            close(output_fd);
                        }
                        let info = exec(command[0].as_str(), &args_addr); //&args_addr == args_addr.as_slice
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
                INFO!("GodOS:/\n$");
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
