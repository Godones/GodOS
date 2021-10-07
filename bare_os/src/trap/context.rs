///保存上下文内容

use riscv::register::{sstatus::{self,Sstatus,SPP}};
#[repr(C)]
pub struct TrapFrame{
    pub reg:[usize;32],//32个通用寄存器
    pub sstatus:Sstatus,
    pub sepc:usize,
}

impl TrapFrame {
    pub fn set_sp(&mut self,sp:usize){
        self.reg[2] = sp;
    }
    pub unsafe fn app_into_context (entry:usize, sp:usize) ->Self{
        let  status = sstatus::read();
        sstatus::set_spp(SPP::User);//将status的spp位置设置为用户态
        let mut tf = Self{
            reg:[0;32],
            sstatus:status,
            sepc:entry,
        };
        tf.set_sp(sp);
        tf
    }
}