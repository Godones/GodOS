extern crate bitflags;

use bitflags::bitflags;

bitflags! {
    pub struct PTEFlags:u8{
        const V = 1<<0;//合法位
        const R = 1<<1;//读
        const W = 1<<2;//写
        const X = 1<<3;//执行
        const U = 1<<4;//处于u特权级是否可用
        const G = 1<<5;
        const A = 1<<6;//访问位
        const D = 1<<7;//修改位
    }
}
pub struct PageTableEntry{
    pub bits:usize//页表项
}