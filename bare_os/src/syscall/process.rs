use alloc::string::String;
// use crate::config::BIG_STRIDE;
use crate::file::{ open_file, OpenFlags};
use crate::mm::address::VirtAddr;
use crate::mm::page_table::{translated_refmut, translated_str, PageTable, translated_ref};
use crate::task::{current_user_token, exit_current_run_next, suspend_current_run_next};
use alloc::sync::Arc;
use alloc::vec::Vec;

const FD_STDOUT: usize = 1;
const FD_STDIN: usize = 2;

use crate::mm::{MapPermission};
use crate::task::processor::{ current_add_area, current_delete_page, current_process};
use crate::timer::Time;

pub fn sys_exit(exit_code: i32) -> ! {
    // INFO!("[kernel] Application exited with code {}", exit_code);
    //函数退出后，运行下一个应用程序
    exit_current_run_next(exit_code);
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
pub fn sys_set_priority(_priority: usize) -> isize {
    //设置应用的特权级
    todo!()
}
pub fn sys_fork() -> isize {
    //拷贝一份
    let current_process = current_process();
    let new_process = current_process.fork();
    let new_pid = new_process.pid.0;
    let process_inner = new_process.get_inner_access();
    //找到主线程
    let task = process_inner.task[0].as_ref().unwrap();
    let trap_cx_ptr = task.get_inner_access().get_trap_cx();
    trap_cx_ptr.reg[10] = 0; //对于子进程来说，其返回值为0
    new_pid as isize //对于父进程来说，其返回值为子进程的pid
}


pub fn sys_exec(path: *const u8, mut args: *const  usize) -> isize {
    //args 里面包含了多个指针，指向多个参数，第一个参数是应用名称的地址
    let token = current_user_token();
    let name = translated_str(token, path);//应用路径
    let mut args_v :Vec<String> = Vec::new();
    loop {
        let arg_ptr = *translated_ref(token,args);//找到第一个参数的指针
        if arg_ptr == 0 { break ; }
        args_v.push(translated_str(token,arg_ptr as *const u8));
        //args_v中字符串已经不包含结束标记\0,且不包含参数的结束标记
        unsafe {
            args = args.add(1);
        }
    }
    if let Some(node) = open_file(name.as_str(), OpenFlags::R) {
        let data = node.read_all();
        // DEBUG!("[kernel] data_size: {:}",data.len());
        let process = current_process();
        let len = args_v.len();
        process.exec(data.as_slice(), args_v);
        len as isize
    } else {
        -1
    }
}
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let current_process = current_process();
    //获取正在执行的进程
    let mut process_inner = current_process.get_inner_access();
    if process_inner
        .children
        .iter()
        .find(|task| pid == -1 || pid as usize == task.get_pid())
        .is_none()
    {
        return -1;
    } //查找是否有对应的子进程或者是pid=-1
    let pair = process_inner
        .children
        .iter()
        .enumerate()
        .find(|(_index, val)| {
            val.get_inner_access().is_zombie() && (pid == -1 || pid as usize == val.get_pid())
        });
    if let Some((idx, _)) = pair {
        //移除子进程
        let child = process_inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1); //确保此时子进程的引用计数为1
        let found_pid = child.get_pid(); //子进程的pid
        let exit_code = child.get_inner_access().exit_code; //
                                                            //向当前执行的进程的保存返回值位置写入子进程的返回值
        *translated_refmut(process_inner.memory_set.token(), exit_code_ptr) = exit_code as i32;
        found_pid as isize //返回找到的子进程pid
    } else {
        -2
    }
}
pub fn sys_spawn(path: *const u8) -> isize {
    //todo!(重新实现spawn)
    //完成新建子进程并执行应用程序的功能，即将exec与fork合并的功能
    //这里的实现是spawn不必像fork一样复制父进程地址空间和内容
    let token = current_user_token();
    let name = translated_str(token, path); //查找是否存在此应用程序
    let process = current_process();
    process.spawn(name.as_str())
    
}
pub fn sys_getpid() -> isize {
    //获取当前进程的pid号
    let current_process = current_process();
    current_process.get_pid() as isize
}
/// 申请长度为 len 字节的物理内存，
/// 将其映射到 start 开始的虚存，内存页属性为 port
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
/// 撤销申请的空间
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



