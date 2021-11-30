use alloc::sync::Arc;
use crate::mm::page_table::{translated_byte_buffer, translated_refmut, translated_str};
use crate::task::{suspend_current_run_next, current_user_token, add_task, exit_current_run_next};
use crate::{print, INFO};
use crate::config::BIG_STRIDE;
use crate::loader::get_data_by_name;
use crate::task::process::copy_current_task;
use crate::sbi::console_getchar;
const FD_STDOUT: usize = 1;
const FD_STDIN:usize = 2;


pub fn sys_exit(exit_code: i32) -> ! {
    INFO!("[kernel] Application exited with code {}", exit_code);
    //函数退出后，运行下一个应用程序
    exit_current_run_next(exit_code as isize);
    panic!("Unreachable sys_exit!")
}
pub fn sys_read(fd:usize,buf:*const u8,len:usize)->isize{
    match fd {
        FD_STDIN=>{
            let mut c :usize;
            loop {
                c = console_getchar();
                if c==0 {
                    suspend_current_run_next();
                    continue;
                }
                else {
                    break
                }
            }
            let ch = c as u8;
            let mut buffer = translated_byte_buffer(current_user_token(),buf,len);
            unsafe {
                //写入用户地址空间的缓冲区中
                buffer[0].as_mut_ptr().write_volatile(ch);
            }
            1
        }
        _ =>{
            panic!("Unsupport fd");
        }
    }
}
pub fn sys_write(function: usize, buf: *const u8, len: usize) -> isize {
    match function {
        FD_STDOUT => {
            let slice = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in slice {
                let str = core::str::from_utf8(buffer).unwrap();
                print!("{}", str);
            }
            len as isize
        }
        _ => {
            //未定义的操作
            panic!("Undefined function in sys_write");
        }
    }
}
pub fn sys_yield() -> isize {
    suspend_current_run_next(); //暂停当前任务运行下一个任务
    0
}
pub fn sys_get_time() -> isize {
    crate::timer::get_costtime() as isize
}
pub fn sys_set_priority(priority: usize) -> isize {
    //设置应用的特权级
    let current_task = copy_current_task().unwrap();
    current_task.get_inner_access().pass = BIG_STRIDE/priority;
    priority as isize
}
pub fn sys_fork()->isize{
    //拷贝一份
    let current_task = copy_current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let trap_cx_ptr= new_task.get_inner_access().get_trap_cx();
    trap_cx_ptr.reg[10] = 0;//对于子进程来说，其返回值为0
    add_task(new_task);
    new_pid as isize //对于父进程来说，其返回值为子进程的pid
}
pub fn sys_exec(path:*const u8)->isize{
    let token = current_user_token();
    let name = translated_str(token,path);
    if let Some(data) = get_data_by_name(name.as_str()){
        let task = copy_current_task().unwrap();
        task.exec(data);
        0
    }
    else {
        -1
    }
}
pub fn sys_waitpid(pid:isize,exit_code_ptr:*mut i32)->isize{
    let current_task = copy_current_task().unwrap();
    //获取正在执行的进程
    let mut task_inner = current_task.get_inner_access();
    if task_inner.children.iter()
        .find(|task|pid==-1||pid as usize==task.get_pid())
        .is_none(){
        return -1;
    }//查找是否有对应的子进程或者是pid=-1
    let pair = task_inner.children.iter()
        .enumerate()
        .find(|(_index,val)|{
            val.get_inner_access().is_zombie()&&(pid==-1||pid as usize==val.get_pid())
        });
    if let Some((idx,_)) = pair{
        //移除子进程
        let  child = task_inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child),1);//确保此时子进程的引用计数为1
        let found_pid = child.get_pid();//子进程的pid
        let exit_code = child.get_inner_access().exit_code;//
        //向当前执行的进程的保存返回值位置写入子进程的返回值
        *translated_refmut(task_inner.memory_set.token(),exit_code_ptr) = exit_code as i32;
        found_pid as isize //返回找到的子进程pid
    }
    else { -2 }

}
pub fn sys_spawn(path:*const u8)->isize{
    //完成新建子进程并执行应用程序的功能，即将exec与fork合并的功能
    //这里的实现是spawn不必像fork一样复制父进程地址空间和内容
    let token = current_user_token();
    let name = translated_str(token,path);//查找是否存在此应用程序
    let task = copy_current_task().unwrap();
    task.spawn(name.as_str());
    -1
}