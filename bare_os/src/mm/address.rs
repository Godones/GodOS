use crate::config::{PAGE_SIZE, PAGE_SIZE_BIT};
use crate::mm::page_table::PageTableEntry;

/// 虚拟地址、物理地址、虚拟页号、物理页帧的定义
#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct VirtAddr(usize);

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct PhysAddr(usize);

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct VirtPageNum(usize);

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq)]
pub struct PhysPageNum(usize);

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        v.0
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}
impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

impl PhysAddr {
    pub fn page_offset(&self) -> usize {
        //地址偏移
        self.0 & (PAGE_SIZE - 1) //通过&运算确定物理地址是否和页大小对齐
    }
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum::from(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum::from((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

impl VirtAddr {
    pub fn page_offset(&self) -> usize {
        //地址偏移
        self.0 & (PAGE_SIZE - 1) //通过&运算确定物理地址是否和页大小对齐
    }
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum::from(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum::from((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
}

impl PhysPageNum {

    //todo
    pub fn get_bytes_array(&self)->&'static mut  [u8]{
        let phyaddress:PhysAddr= self.0.clone().into();//获得物理地址
        unsafe {
            core::slice::from_raw_parts_mut(phyaddress.0 as *mut u8,4096)
        }
    }
}

impl From<PhysAddr> for PhysPageNum {
    //从物理地址到物理页号
    fn from(v: PhysAddr) -> Self {
        // assert_eq!(v.page_offset(),0);//是否对齐
        v.floor()
    }
}

impl From<VirtAddr> for VirtPageNum {
    //虚拟地址 --> 物理地址
    fn from(v: VirtAddr) -> Self {
        // assert_eq!(v.page_offset(),0);
        v.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    //从物理页号转换位物理地址只需要左移12位即可
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BIT)
    }
}

impl From<VirtPageNum> for VirtAddr {
    //虚拟页号-->虚拟地址
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BIT)
    }
}
