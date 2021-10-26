use crate::println;
use crate::config::{PAGE_SIZE,PAGE_SIZE_BIT};

/// 虚拟地址、物理地址、虚拟页号、物理页帧的定义
#[derive(Copy, Clone,PartialEq,Ord, PartialOrd, Eq)]
struct VirtAddr(usize);

#[derive(Copy, Clone,PartialEq,Ord, PartialOrd, Eq)]
struct PhysAddr(usize);

#[derive(Copy, Clone,PartialEq,Ord, PartialOrd, Eq)]
struct VirtPageNum(usize);

#[derive(Copy, Clone,PartialEq,Ord, PartialOrd, Eq)]
struct PhysPageNum(usize);

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self { Self(value)}
}
impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self { Self(value) }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self { Self(value) }
}

impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self { Self(value) }
}

impl PhysAddr {
    pub fn page_offset(&self)->usize{
        //地址偏移
        self.0&(PAGE_SIZE-1) //通过&运算确定物理地址是否和页大小对齐
    }
    pub fn floor(&self)->PhysPageNum{ PhysPageNum::from(self.0/PAGE_SIZE)};
    pub fn ceil(&self)->PhysPageNum{
        PhysPageNum::from((self.0 + PAGE_SIZE - 1)/PAGE_SIZE)
    }
}

impl VirtAddr {
    pub fn page_offset(&self)->usize{
        //地址偏移
        self.0&(PAGE_SIZE-1) //通过&运算确定物理地址是否和页大小对齐
    }
    pub fn floor(&self)->VirtPageNum{ VirtPageNum::from(self.0/PAGE_SIZE)};
    pub fn ceil(&self)->VirtPageNum{
        VirtPageNum::from((self.0 + PAGE_SIZE - 1)/PAGE_SIZE)
    }
}


impl From<PhysAddr> for PhysPageNum {
    fn from(PhysAddr: PhysAddr) -> Self {
        assert_eq!(PhysAddr.page_offset(),0);//是否对齐
        PhysAddr.floor()
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(),0);
        v.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    //从物理页号转换位物理地址只需要左移12位即可
    fn from(PhysPageNum: PhysPageNum) -> Self {
        Self(PhysPageNum<<PAGE_SIZE_BIT)
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(VirtPageNum<<PAGE_SIZE_BIT)
    }
}
