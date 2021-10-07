
use crate::println;
use core::cell::RefCell;
use core::slice::from_raw_parts;
use lazy_static::lazy_static;
use crate::trap::context::TrapFrame;
/// 应用管理器，找到并加载应用程序的二进制文件
const MAX_APP_NUM: usize = 10;
const APP_BASE_ADDRESS: usize = 0x80400000; //应用程序起始地址
const APP_SIZE_LIMIT: usize = 0x20000; //应用程序的空间限制
const USER_STACK_SIZE:usize = 4096*2;//用户栈大小
const KERNEL_STACK_SIZE:usize = 4096*2;//内核栈大小

#[repr(align(4096))]
struct KernelStack{
    data:[u8;KERNEL_STACK_SIZE]
}
#[repr(align(4096))]
struct UserStack{
    data:[u8;USER_STACK_SIZE],
}

static KERNEL_STACK:KernelStack = KernelStack{data:[0;KERNEL_STACK_SIZE]};
static USER_STACK:UserStack = UserStack{data:[0;USER_STACK_SIZE]};

impl UserStack {
    //获取栈顶地址
    fn get_sp(&self)->usize{
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

impl KernelStack {
    fn get_sp(&self)->usize{
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self,cx:TrapFrame)->&'static mut TrapFrame{
        let cx_ptr = (self.get_sp()-core::mem::size_of::<TrapFrame>() )as *mut TrapFrame;
        unsafe {
            *cx_ptr = cx;
            cx_ptr.as_mut().unwrap()
        }
    }
}

struct AppManager {
    inner: RefCell<AppManagerInner>,
}
struct AppManagerInner {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

unsafe impl Sync for AppManager {}

lazy_static! {
    static ref APP_MANAGER: AppManager = AppManager{
        inner: RefCell::new({
            extern "C"{ fn _num_app();}
            //取出app所在位置的地址
            //link_apps按8字节对其
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = unsafe{ num_app_ptr.read_volatile()};//=3
            let mut app_start :[usize;MAX_APP_NUM+1] = [0;MAX_APP_NUM+1];
            let app_start_raw:&[usize] = unsafe{
                //形成一个指针切片，存放的三个应用的起始地址和最后一个应用的开始地址
                from_raw_parts(num_app_ptr.add(1),num_app+1)
            };
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManagerInner{
                num_app,
                current_app:0,
                app_start,
            }
        }),
    };
}
impl AppManagerInner {
    fn print_app_info(&self) {
        println!("[kernel] app_{}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#?}, {:#?}]",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }
    fn get_current_app(&self) -> usize {
        self.current_app
    }
    fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("All application completed!");
        }
        println!("[kernel] Loading app_{}", app_id);

        //重要 clear i-cache
        asm!("fence.i", options(nostack));
        //清除应用程序段
        (APP_BASE_ADDRESS..APP_BASE_ADDRESS + APP_SIZE_LIMIT).for_each(|addr| {
            (addr as *mut u8).write_volatile(0); //取地址并写入0
        });
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src); //写入数据
    }
}
pub fn print_app_info() {
    APP_MANAGER.inner.borrow().print_app_info();
}
pub fn init() {
    print_app_info();
}





pub fn run_next_app() ->!{
    let current_app = APP_MANAGER.inner.borrow().current_app;
    unsafe {
        APP_MANAGER.inner.borrow().load_app(current_app);
    }
    APP_MANAGER.inner.borrow_mut().move_to_next_app();
    extern "C"{
        fn _restore(cx_addr:usize);
    }
    unsafe {
        _restore(KERNEL_STACK.push_context(
            TrapFrame::app_into_context(APP_BASE_ADDRESS,
            USER_STACK.get_sp()))as * const _ as usize
        );
    }
    panic!("The end of application");
}
