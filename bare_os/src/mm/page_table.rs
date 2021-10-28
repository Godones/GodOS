extern crate bitflags;

use crate::mm::address::PhysPageNum;
use bitflags::bitflags;
//页表项标志位
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
#[derive(Copy, Clone)]
pub struct PageTableEntry {
    pub bits: usize, //页表项
}

impl PageTableEntry {
    fn new(ppn: usize, flags: PTEFlags) -> Self {
        //根据物理页号和标志位创建一个页表项
        PageTableEntry {
            bits: ppn << 10 | flags.bits as usize,
        }
    }
    fn empty() -> Self {
        //获取一个空项,不合法的项
        PageTableEntry { bits: 0 }
    }
    fn ppn(&self) -> PhysPageNum {
        //ppn占据 10-53位
        (self.bits >> 10 & (1usize << 44 - 1)).into()
    }
    fn flags(&self) -> PTEFlags {
        //截断低8位
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    fn is_valid(&self) -> bool {
        //是否有效,即V标志位是否位1
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}
