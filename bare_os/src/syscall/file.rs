use alloc::sync::Arc;
use crate::file::{create_nlink_file, delete_nlink_file, open_file, OpenFlags, Pipe, Stat};
use crate::{list_apps};
use crate::mm::page_table::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};

use crate::task::current_user_token;
use crate::task::processor::current_process;

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let current_process_inner = process.get_inner_access();
    if fd < current_process_inner.fd_table.len() {
        match &current_process_inner.fd_table[fd] {
            Some(file) => {
                let file = file.clone();
                drop(current_process_inner);
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
    let process = current_process();
    let current_process_inner = process.get_inner_access();
    if fd < current_process_inner.fd_table.len() {
        match &current_process_inner.fd_table[fd] {
            Some(file) => {
                let file = file.clone();
                drop(current_process_inner);
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
    if let Some(node) = open_file(name.as_str(),OpenFlags::from_bits(flags).unwrap()){
        let process = current_process();
        let mut inner = process.get_inner_access();
        // let data = node.read_all();
        // DEBUG!("[kernel-sys-open] data:{:.2}",data.len() as f64/1024 as f64);
        let fd = inner.get_one_fd();//分配文件描述符
        //
        inner.fd_table[fd] = Some(node.clone());
        fd as isize
    }
    else {
        -1
    }
}


pub fn sys_pipe(pipe: *mut usize) -> isize {
    let token = current_user_token();
    let current_process = current_process();
    let mut inner = current_process.get_inner_access();
    // DEBUG!("[kernel] sys_pipe");
    let (read_end, write_end) = Pipe::new(); //声请两个文件
    let fd_read_end = inner.get_one_fd();
    inner.fd_table[fd_read_end] = Some(read_end);
    let fd_write_end = inner.get_one_fd();
    inner.fd_table[fd_write_end] = Some(write_end);
    *translated_refmut(token, pipe) = fd_read_end;
    *translated_refmut(token, unsafe { pipe.add(1) }) = fd_write_end;
    0
}
pub fn sys_close(fd: usize) -> isize {
    // "关闭进程打开的文件描述符
    let process = current_process();
    let mut process_inner = process.get_inner_access();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        return -1; //检查是否已经关闭过
    }
    process_inner.fd_table[fd].take();
    0
}
pub fn sys_mail_read(buf:*mut u8,len:usize)->isize{
    sys_read(3, buf, len)
}
pub fn sys_mail_write(_fd:usize,buf:*mut u8,len:usize)->isize{
    //todo!(需要修改pid的查找)
    sys_write(3, buf, len)
}


pub fn sys_dup(fd:usize)->isize{
    let process = current_process();
    let mut inner = process.get_inner_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }else if inner.fd_table[fd].is_none(){
        return -1;
    }
    let new_fd = inner.get_one_fd();
    inner.fd_table[new_fd] =Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));//复制fd
    new_fd as isize
}
pub fn sys_ls()->isize{
    list_apps();
    0
}

///根据fd找到文件的相关信息
pub fn sys_fstat(fd:usize,stat:*mut Stat)->isize{
    let token = current_user_token();
    let current_process = current_process();
    let current_process_inner = current_process.get_inner_access();
    if fd < current_process_inner.fd_table.len() {
        match &current_process_inner.fd_table[fd] {
            Some(file) => {
                let file = file.clone();
                drop(current_process_inner);
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
    let old_path = translated_str(token,old_path);//已经没有'\0'结束标记
    let new_path = translated_str(token,new_path);
    create_nlink_file(new_path.as_str(),old_path.as_str())
}
///解除硬链接
pub fn sys_unlinkat(path:*const u8)->isize{
    let token = current_user_token();
    let path = translated_str(token,path);
    delete_nlink_file(path.as_str())
}