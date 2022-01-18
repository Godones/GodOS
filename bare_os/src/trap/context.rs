///保存上下文内容
use riscv::register::sstatus::{self, Sstatus, SPP};
#[repr(C)]
pub struct TrapFrame {
    pub reg: [usize; 32], //32个通用寄存器
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,  //内核的地址空间根页表位置
    pub kernel_sp: usize,    //内核的用户栈栈顶 位置
    pub trap_handler: usize, //内核处理trap的位置
}

impl TrapFrame {
    pub fn set_sp(&mut self, sp: usize) {
        //x2寄存器
        self.reg[2] = sp;
    }
    pub fn app_into_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_stack_sp: usize,
        trap_handler: usize,
    ) -> Self {
        //为启动应用程序而特殊构造的 Trap 上下文，
        //sp: 用户栈顶地址
        unsafe {
            sstatus::set_spp(SPP::User);
        } //将status的spp位置设置为用户态
        let status = sstatus::read();
        let mut trap_cx = Self {
            reg: [0; 32],
            sstatus: status,
            sepc: entry,              //动态链接的应用程序入口
            kernel_satp,              //内核的satp
            kernel_sp: kernel_stack_sp, //应用程序在内核的栈顶地址
            trap_handler,
        };
        trap_cx.set_sp(sp);
        trap_cx
    }
}
