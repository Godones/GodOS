
use crate::file::{ create_nlink_file, delete_nlink_file, open_file, OpenFlags, Stat};
use crate::mm::page_table::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};

use crate::task::current_user_token;
use crate::task::process::copy_current_task;

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let current_task = copy_current_task().unwrap();
    let current_task_inner = current_task.get_inner_access();
    if fd < current_task_inner.fd_table.len() {
        match &current_task_inner.fd_table[fd] {
            Some(file) => {
                let file = file.clone();
                drop(current_task_inner);
                let buffer = translated_byte_buffer(token, buf, len);
                file.read(UserBuffer::new(buffer)) as isize
            }
            _ => -1,
        }
    } else {
        return -1; //不存在打开的文件
    }
}
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let current_task = copy_current_task().unwrap();
    let current_task_inner = current_task.get_inner_access();
    if fd < current_task_inner.fd_table.len() {
        match &current_task_inner.fd_table[fd] {
            Some(file) => {
                let file = file.clone();
                drop(current_task_inner);
                let buffer = translated_byte_buffer(token, buf, len);
                file.write(UserBuffer::new(buffer)) as isize
            }
            _ => -1,
        }
    } else {
        return -1; //不存在打开的文件
    }
}
pub fn sys_open(path:*const u8,flags:u32)->isize{
    //打开文件返回一个描述符
    let token = current_user_token();
    let name = translated_str(token,path);
    // println!("the file name: {}",name);
    if let Some(node) = open_file(name.as_str(),OpenFlags::from_bits(flags).unwrap()){
        let task = copy_current_task().unwrap();
        let mut inner = task.get_inner_access();
        // let data = node.read_all();
        // DEBUG!("size:{}",node.get_file_size());
        // DEBUG!("[kernel-sys-open] data:{} {}",data.len(),core::str::from_utf8(data.as_slice()).unwrap());
        let fd = inner.get_one_fd();//分配文件描述符
        //
        inner.fd_table[fd] = Some(node.clone());
        fd as isize
    }
    else {
        -1
    }
}
///根据fd找到文件的相关信息
pub fn sys_fstat(fd:usize,stat:*mut Stat)->isize{
    let token = current_user_token();
    let current_task = copy_current_task().unwrap();
    let current_task_inner = current_task.get_inner_access();
    if fd < current_task_inner.fd_table.len() {
        match &current_task_inner.fd_table[fd] {
            Some(file) => {
                let file = file.clone();
                drop(current_task_inner);
                let fstat = file.fstat();
                 *translated_refmut(token,stat) = fstat;
                0
            }
            _ => -1,
        }
    } else {
        return -1; //不存在打开的文件
    }
}
///建立硬链接
pub fn sys_linkat(old_path:*const u8,new_path:*const u8)->isize{
    let token = current_user_token();
    let old_path = translated_str(token,old_path);
    let new_path = translated_str(token,new_path);
    create_nlink_file(new_path.as_str(),old_path.as_str())
}
///解除硬链接
pub fn sys_unlinkat(path:*const u8)->isize{
    let token = current_user_token();
    let path = translated_str(token,path);
    delete_nlink_file(path.as_str())
}