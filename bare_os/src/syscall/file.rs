use crate::mm::page_table::{translated_byte_buffer, UserBuffer};
use crate::println;
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
