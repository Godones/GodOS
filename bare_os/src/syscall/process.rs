
use crate::config::BIG_STRIDE;
use crate::file::{open_file, Pipe,OpenFlags};
use crate::mm::address::VirtAddr;
use crate::mm::page_table::{translated_refmut, translated_str, PageTable};
use crate::task::{add_task, current_user_token, exit_current_run_next, suspend_current_run_next};
use alloc::sync::Arc;

const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 2;

use crate::mm::MapPermission;
use crate::task::process::{copy_current_task, current_add_area, current_delete_page};
use crate::timer::Time;

use super::file::{sys_read, sys_write};

pub fn sys_exit(exit_code: i32) -> ! {
    // INFO!("[kernel] Application exited with code {}", exit_code);
    //函数退出后，运行下一个应用程序
    exit_current_run_next(exit_code as isize);
    panic!("Unreachable sys_exit!")
}

pub fn sys_yield() -> isize {
    suspend_current_run_next(); //暂停当前任务运行下一个任务
    0
}
pub fn sys_get_time(time: *mut Time) -> isize {
    let current_time = crate::timer::get_costtime(); //获取微秒
                                                     // println!("current: {}",current_time);
    *(translated_refmut(current_user_token(), time)) = Time {
        s: current_time / 1000_000,
        us: current_time % 1000_000,
    };

    0
}
pub fn sys_set_priority(priority: usize) -> isize {
    //设置应用的特权级
    let current_task = copy_current_task().unwrap();
    current_task.get_inner_access().pass = BIG_STRIDE / priority;
    priority as isize
}
pub fn sys_fork() -> isize {
    //拷贝一份
    let current_task = copy_current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let trap_cx_ptr = new_task.get_inner_access().get_trap_cx();
    trap_cx_ptr.reg[10] = 0; //对于子进程来说，其返回值为0
    add_task(new_task);
    new_pid as isize //对于父进程来说，其返回值为子进程的pid
}
pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let name = translated_str(token, path);

    if let Some(node) = open_file(name.as_str(),OpenFlags::R) {
        let data = node.read_all();
        let task = copy_current_task().unwrap();
        task.exec(data.as_slice());
        0
    } else {
        -1
    }
}
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let current_task = copy_current_task().unwrap();
    //获取正在执行的进程
    let mut task_inner = current_task.get_inner_access();
    if task_inner
        .children
        .iter()
        .find(|task| pid == -1 || pid as usize == task.get_pid())
        .is_none()
    {
        return -1;
    } //查找是否有对应的子进程或者是pid=-1
    let pair = task_inner
        .children
        .iter()
        .enumerate()
        .find(|(_index, val)| {
            val.get_inner_access().is_zombie() && (pid == -1 || pid as usize == val.get_pid())
        });
    if let Some((idx, _)) = pair {
        //移除子进程
        let child = task_inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1); //确保此时子进程的引用计数为1
        let found_pid = child.get_pid(); //子进程的pid
        let exit_code = child.get_inner_access().exit_code; //
                                                            //向当前执行的进程的保存返回值位置写入子进程的返回值
        *translated_refmut(task_inner.memory_set.token(), exit_code_ptr) = exit_code as i32;
        found_pid as isize //返回找到的子进程pid
    } else {
        -2
    }
}
pub fn sys_spawn(path: *const u8) -> isize {
    //完成新建子进程并执行应用程序的功能，即将exec与fork合并的功能
    //这里的实现是spawn不必像fork一样复制父进程地址空间和内容
    let token = current_user_token();
    let name = translated_str(token, path); //查找是否存在此应用程序
    let task = copy_current_task().unwrap();
    task.spawn(name.as_str())
    
}

pub fn sys_getpid() -> isize {
    //获取当前进程的pid号
    let current_task = copy_current_task().unwrap();
    current_task.get_pid() as isize
}

// 申请长度为 len 字节的物理内存，
// 将其映射到 start 开始的虚存，内存页属性为 port
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    let start_vir: VirtAddr = start.into(); //与页大小对齐
                                            //除了低8位其它位必须为0;
                                            //低8位不能全部为0
    if start_vir.aligned() != true || (port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    //判断是否已经存在某个页被映射
    let new_port: u8 = (port & 0x7) as u8;
    let permission = MapPermission::U;
    let map_permission = MapPermission::from_bits(new_port << 1 as u8).unwrap() | permission;

    let start_vpn = start_vir.floor(); //起始页
    let end_vpn = VirtAddr::from(start + len).ceil(); //向上取整结束页

    //申请到一个map_area后判断其每个页是否出现在map_area中过
    let current_user_token = current_user_token(); //获取当前用户程序的satp
    let temp_page_table = PageTable::from_token(current_user_token);
    for vpn in start_vpn.0..end_vpn.0 {
        if let Some(_val) = temp_page_table.find_pte(vpn.into()) {
            return -1;
        } //提前返回错误值
    }
    // let map_area = MapArea::new(start_vir,(start+len).into(),Framed,map_permission);
    current_add_area(start_vir, (start + len).into(), map_permission);
    0
}
//撤销申请的空间
pub fn sys_munmap(start: usize, len: usize) -> isize {
    let start_vir: VirtAddr = start.into(); //与页大小对齐
    if !start_vir.aligned() {
        return -1;
    }
    // DEBUG!("[kernel] here");
    let start_vpn = start_vir.floor(); //起始页
    let end_vpn = VirtAddr::from(start + len).ceil(); //向上取整结束页
    let current_user_token = current_user_token(); //获取当前用户程序的satp
    let temp_page_table = PageTable::from_token(current_user_token);
    for vpn in start_vpn.0..end_vpn.0 {
        if temp_page_table.find_pte(vpn.into()).is_none() {
            return -1;
        } //提前返回错误值,如果这些页存在不位于内存的则错误返回
    }
    current_delete_page(start_vir);
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let token = current_user_token();
    let current_task = copy_current_task().unwrap();
    let mut inner = current_task.get_inner_access();
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
    let task = copy_current_task().unwrap();
    let mut task_inner = task.get_inner_access();
    if fd >= task_inner.fd_table.len() {
        return -1;
    }
    if task_inner.fd_table[fd].is_none() {
        return -1; //检查是否已经关闭过
    }
    task_inner.fd_table[fd].take();
    0
}

pub fn sys_mail_read(buf:*mut u8,len:usize)->isize{
    sys_read(3, buf, len)
}

pub fn sys_mail_write(pid:usize,buf:*mut u8,len:usize)->isize{
    //todo!(需要修改pid的查找)
    sys_write(3, buf, len)
}
pub fn sys_open(path:*const u8,flags:u32)->isize{
    //打开文件返回一个描述符
    let token = current_user_token();
    let name = translated_str(token,path);
    if let Some(node) = open_file(name.as_str(),OpenFlags::from_bits(flags).unwrap()){
        let task = copy_current_task().unwrap();
        let mut inner = task.get_inner_access();
        let fd = inner.get_one_fd();//分配文件描述符
        //
        inner.fd_table[fd] = Some(node);
        fd as isize
    }
    else {
        -1
    }

}