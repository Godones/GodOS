///! 保存上下文内容

use riscv::register::{sstatus::Sstatus,scause::Scause};
#[repr(C)]
pub struct TrapFrame{
    pub reg:[usize;32],//32个通用寄存器
    pub sstatus:Sstatus,
    pub sepc:usize,
    pub stval:usize,
    pub scause:Scause,
}

