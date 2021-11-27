use core::task::Poll;
use crate::mm::page_table::{PageTable, translated_refmut};
use crate::task::{current_add_area, current_delete_page, set_priority};
use crate::task::suspend_current_run_next;
use crate::task::{current_user_token, exit_current_run_next};
use crate::timer::Time;
use crate::{print, INFO, println, DEBUG};
use crate::config::PAGE_SIZE;
use crate::mm::address::VirtAddr;
use crate::mm::MapPermission;
use crate::mm::memory_set::MapArea;
use crate::mm::memory_set::MapType::Framed;

const FUNCTION_STDOUT: usize = 1;
pub fn sys_exit(xstate: i32) -> ! {
    INFO!("[kernel] Application exited with code {}", xstate);
    //函数退出后，运行下一个应用程序
    exit_current_run_next();
    panic!("Unreachable sys_exit!")
}

pub fn sys_write(function: usize, buf: *const u8, len: usize) -> isize {
    match function {
        FUNCTION_STDOUT => {
            let slice = PageTable::translated_byte_buffer(current_user_token(), buf, len);
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
pub fn sys_get_time(time: *mut Time) -> isize {
    let current_time = crate::timer::get_cost_time(); //获取微秒
    // println!("current: {}",current_time);
    unsafe {
        *(translated_refmut(current_user_token(),time)) = Time {
            s: current_time / 1000_000,
            us: current_time % 1000_000,
        };
    }
    0
}
// 申请长度为 len 字节的物理内存，
// 将其映射到 start 开始的虚存，内存页属性为 port
pub fn sys_mmap(start:usize,len:usize,port:usize)->isize{
    let start_vir:VirtAddr = start.into();//与页大小对齐
    //除了低8位其它位必须为0;
    //低8位不能全部为0
    if start_vir.aligned()!=true || (port&!0x7!=0)|| (port&0x7==0) {
        return  -1;
    }
    //判断是否已经存在某个页被映射
    let new_port :u8 = (port&0x7) as u8;
    let permission = MapPermission::U;
    let map_permission = MapPermission::from_bits( new_port<<1 as u8).unwrap()|permission;

    let start_vpn = start_vir.floor();//起始页
    let end_vpn = VirtAddr::from(start+len).ceil();//向上取整结束页

    //申请到一个map_area后判断其每个页是否出现在map_area中过
    let current_user_token = current_user_token();//获取当前用户程序的satp
    let temp_page_table = PageTable::from_token(current_user_token);
    for vpn in start_vpn.0..end_vpn.0{
        if let Some(val) = temp_page_table.find_pte(vpn.into()){
            return -1;
        }//提前返回错误值
    }
    let map_area = MapArea::new(start_vir,(start+len).into(),Framed,map_permission);
    current_add_area(map_area);
    0
}
//撤销申请的空间
pub fn sys_munmap(start:usize,len:usize)->isize{
    let start_vir:VirtAddr = start.into();//与页大小对齐
    if !start_vir.aligned() {
        return -1;
    }
    DEBUG!("[kernel] here");
    let start_vpn = start_vir.floor();//起始页
    let end_vpn = VirtAddr::from(start+len).ceil();//向上取整结束页
    let current_user_token = current_user_token();//获取当前用户程序的satp
    let temp_page_table = PageTable::from_token(current_user_token);
    for vpn in start_vpn.0..end_vpn.0{
        if temp_page_table.find_pte(vpn.into()).is_none(){
            return -1;
        }//提前返回错误值,如果这些页存在不位于内存的则错误返回
    }
    for vpn in start_vpn.0..end_vpn.0 {
        current_delete_page(vpn.into());
    }
    0
}

pub fn sys_set_priority(priority: usize) -> isize {
    //设置应用的特权级
    set_priority(priority)
}
