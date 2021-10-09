
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


static KERNEL_STACK:KernelStack = KernelStack{data:[0;KERNEL_STACK_SIZE]};
static USER_STACK:UserStack = UserStack{data:[0;USER_STACK_SIZE]};



#[repr(align(4096))]
struct KernelStack{
    data:[u8;KERNEL_STACK_SIZE]
}
#[repr(align(4096))]
struct UserStack{
    data:[u8;USER_STACK_SIZE],
}


impl UserStack {
    //获取栈顶地址
    fn get_sp(&self)->usize{
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

impl KernelStack {
    //获取内核栈栈顶地址
    fn get_sp(&self)->usize{
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self,cx:TrapFrame)->&'static mut TrapFrame{
        //在内核栈上压入trap上下文
        let cx_ptr = (self.get_sp()-core::mem::size_of::<TrapFrame>() )as *mut TrapFrame;
        unsafe {
            *cx_ptr = cx;
            cx_ptr.as_mut().unwrap()
            //返回内核栈地址
        }
    }
}

struct AppManager {
    inner: RefCell<AppManagerInner>,
}
struct AppManagerInner {
    num_app: usize,//app数量
    current_app: usize, //当前的app
    app_start: [usize; MAX_APP_NUM + 1], //app的起始地址
}

unsafe impl Sync for AppManager {}

lazy_static! {
    ///初始化app 从link_app.S找到_num_app,
    /// 并从此处开始解析应用程序数量和各个应用程序的地址
     static ref APP_MANAGER: AppManager = AppManager{
        inner: RefCell::new({
            extern "C"{ fn _num_app();}
            //取出app所在位置的地址
            //link_apps按8字节对其
            let num_app_ptr = _num_app as usize as *const usize;//取地址
            let num_app = unsafe{ num_app_ptr.read_volatile()};//读内容 =3
            let mut app_start :[usize;MAX_APP_NUM+1] = [0;MAX_APP_NUM+1];
            let app_start_raw:&[usize] = unsafe{
                //形成一个指针切片，存放的三个应用的起始地址和最后一个应用的开始地址
                from_raw_parts(num_app_ptr.add(1),num_app+1)
            };
            app_start[..=num_app].copy_from_slice(app_start_raw);//复制地址
            AppManagerInner{
                num_app,
                current_app:0,
                app_start,
            }//初始化
        }),
    };
}
impl AppManagerInner {
    /// 打印app相关信息
    /// 包括正在app数量， 各个app的起始地址
    fn print_app_info(&self) {
        println!("[kernel] app_num: {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [0x{:X}, 0x{:X}]",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }
    ///返回当前的application
    fn get_current_app(&self) -> usize {
        self.current_app
    }
    ///运行下一个application
    fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
    ///加载app
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            panic!("All application completed!");
        }
        println!("[kernel] Loading app_{}", app_id);
        //重要 clear i-cache
        asm!(
            "fence.i",
            options(nostack)
        );

        //清除应用程序段
        (APP_BASE_ADDRESS..APP_BASE_ADDRESS + APP_SIZE_LIMIT).for_each(|addr| {
            (addr as *mut u8).write_volatile(0); //取地址并写入0，以字节写入
        });
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8,//起始地址
            self.app_start[app_id + 1] - self.app_start[app_id],//长度，以字节记
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        app_dst.copy_from_slice(app_src); //写入数据
        // println!("[kernel] batch write data over");
    }
}
fn print_app_info() {
    APP_MANAGER.inner.borrow().print_app_info();
}

///暴露给外部的接口
/// 打印相关信息，并且这个时候APP_MANAGER初始化完成
pub fn init() {
    print_app_info();
}
//下一个app
pub fn run_next_app() ->!{
    let current_app = APP_MANAGER.inner.borrow().current_app;
    unsafe {
        APP_MANAGER.inner.borrow().load_app(current_app);//加载application到0x80400000位置开始运行
    }
    //设置下一个应用
    APP_MANAGER.inner.borrow_mut().move_to_next_app();
    extern "C"{
        fn _restore(cx_addr:usize); //定义外部接口，来自trap.asm用于恢复上下文
    }
    // 复用_restore函数
    // 在内核栈上压入一个Trap上下文
    // sepc 是应用程序入口地址 0x80400000 ，
    // 其 sp 寄存器指向用户栈，其sstatus 的 SPP 字段被设置为 User 。
    // push_context 的返回值是内核栈压入 Trap 上下文之后的内核栈顶，
    // 它会被作为 __restore 的参
    unsafe {
        // println!("[kernel] Begin run application!");
        _restore(KERNEL_STACK.push_context(
            TrapFrame::app_into_context(
                APP_BASE_ADDRESS,
                USER_STACK.get_sp()))as * const _ as usize
        );
        //此时sp指向的是用户栈地址，sscratch指向的是内核栈地址
    }
    panic!("The end of application");
}
