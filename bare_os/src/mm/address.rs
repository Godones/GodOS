use crate::config::{PAGE_SIZE, PAGE_SIZE_BIT};
use crate::mm::page_table::PageTableEntry;

/// 虚拟地址、物理地址、虚拟页号、物理页帧的定义
#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Debug)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Debug)]
pub struct VirtPageNum(pub usize);

#[derive(Copy, Clone, PartialEq, Ord, PartialOrd, Eq, Debug)]
pub struct PhysPageNum(pub usize);

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
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe {
            (self.0 as *mut T).as_mut().unwrap()
        }
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

impl VirtPageNum {
    pub fn index(&self) -> [usize; 3] {
        let mut pagenum = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = pagenum & 511; //取出低9位
            pagenum >>= 9;
        }
        idx
    }
}

//虚拟页号迭代器？
pub trait StepByOne {
    fn step(&mut self);
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VirPageIter {
    current: VirtPageNum,
    l: VirtPageNum,
    r: VirtPageNum,
}

impl VirPageIter {
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        assert!(start.0 <= end.0);
        Self {
            current: start,
            l: start,
            r: end,
        }
    }
    pub fn get_start(&self) -> VirtPageNum {
        self.l
    }
    pub fn get_end(&self) -> VirtPageNum {
        self.r
    }
}

impl StepByOne for VirPageIter {
    fn step(&mut self) {
        self.current.step();
    }
}
impl Iterator for VirPageIter {
    type Item = VirtPageNum;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 < self.r.0 {
            let value = Some(self.current);
            self.current.step();
            value
        } else {
            None
        }
    }
}
pub type VPNRange = VirPageIter;

impl PhysPageNum {
    //以不同方式访问一个物理页帧
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        //将物理页帧转换为字节数组，方便进行写操作
        let phyaddress: PhysAddr = self.clone().into(); //获得物理地址
        unsafe { core::slice::from_raw_parts_mut(phyaddress.0 as *mut u8, 4096) }
    }
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        //返回物理页帧中所有的页表项
        let phyaddress: PhysAddr = self.clone().into();
        unsafe { core::slice::from_raw_parts_mut(phyaddress.0 as *mut PageTableEntry, 512) }
    }
    pub fn get_mut<T>(&self) -> &'static mut T {
        let phyaddress: PhysAddr = self.clone().into();
        phyaddress.get_mut()
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
