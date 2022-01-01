extern crate bitflags;

use crate::mm::address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::iter::IntoIterator;
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
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize, //页表项
}

impl PageTableEntry {
    fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        //根据物理页号和标志位创建一个页表项
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    fn empty() -> Self {
        //获取一个空项,不合法的项
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        //ppn占据 10-53位
        // println!("[Debug] ppn: {}",self.bits);
        let answer: PhysPageNum = (self.bits >> 10 & ((1usize << 44) - 1)).into();
        // println!("[Debug] PhysPageNum: {:?}",answer);
        answer
    }
    pub fn flags(&self) -> PTEFlags {
        //截断低8位
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        //是否有效,即V标志位是否位1
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

pub struct PageTable {
    root_ppn: PhysPageNum,     //根页表所在的物理页帧号
    frames: Vec<FrameTracker>, //所有级别的页表所在的物理页帧
}

impl PageTable {
    pub fn new() -> Self {
        //为根页表申请一个物理页帧
        let root_frame = frame_alloc().unwrap();

        PageTable {
            root_ppn: root_frame.ppn,
            frames: vec![root_frame],
        }
    }
    pub fn token(&self) -> usize {
        //构造一个satp数据
        //最高四位 为模式选择，低44位是根目录所在的物理页帧号
        8usize << 60 | self.root_ppn.0
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        //添加一个虚拟页号到物理页号的映射
        let pte = self.find_pte_create(vpn).unwrap();
        //查找虚拟页号是否已经被映射过了
        assert!(!pte.is_valid(), "vpn: {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V); //建立一个映射
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        //删除一个虚拟页号对应的页表项
        // todo!(应该使用find_pte)
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(pte.is_valid(), "vpn: {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty(); //空项
    }
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        //根据虚拟页号找到页表项
        let idxs = vpn.index(); //将虚拟页表号划分3部分
                                // DEBUG!("[Debug] idxs: {:?} root_ppn: {:?}",idxs,self.root_ppn);
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..3 {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == 2 {
                result = Some(pte);
                return result;
            }
            if !pte.is_valid() {
                let new_frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(new_frame.ppn, PTEFlags::V);
                self.frames.push(new_frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    //下方的代码用来手动查
    // 找页表项

    pub fn from_token(stap: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(stap & ((1 << 44) - 1)),
            frames: Vec::new(),
        }
    }
    pub fn translated_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        //将一个虚拟地址转换为一个物理地址
        self.find_pte(va.floor()).map(|pte| {
            //找到对应的页表项
            let phyaddr: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let align_phyaddr: usize = phyaddr.into();
            (align_phyaddr + offset).into()
        })
    }

    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&PageTableEntry> {
        //根据虚拟页号找到页表项
        let idxs = vpn.index(); //将虚拟页表号划分
        let mut ppn = self.root_ppn;
        let mut result: Option<&PageTableEntry> = None;
        for i in 0..3 {
            let pte = &ppn.get_pte_array()[idxs[i]];

            if !pte.is_valid() {
                return None;
            }
            if i == 2 {
                result = Some(pte);
                return result;
            }

            ppn = pte.ppn();
        }
        result
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| pte.clone())
    }
}

pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    //在内核打印字符时需要访问用户地址空间缓冲区的内容
    let page_table = PageTable::from_token(token);
    let mut start_addr = ptr as usize; //起始地址
    let end = start_addr + len; //结束地址
    let mut contents = Vec::new();

    while start_addr < end {
        let start_viraddr = VirtAddr::from(start_addr);
        let mut vpn = start_viraddr.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_viraddr: VirtAddr = vpn.into();
        end_viraddr = end_viraddr.min(VirtAddr::from(end));

        if end_viraddr.page_offset() == 0 {
            contents.push(&mut ppn.get_bytes_array()[start_viraddr.page_offset()..]);
        } else {
            contents.push(
                &mut ppn.get_bytes_array()[start_viraddr.page_offset()..end_viraddr.page_offset()],
            );
        }

        start_addr = end_viraddr.into();
    }
    contents
}
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    //根据token和字符串指针在应用地址空间中查找
    //应用程序的名称
    let page_table = PageTable::from_token(token);
    let mut name = String::new();
    let mut start = ptr as usize;
    loop {
        let ch: u8 = *(page_table
            .translated_va(VirtAddr::from(start))
            .unwrap()
            .get_mut()); //将虚拟地址转化为物理地址，再从物理地址取出相应的内容
        if ch == 0 {
            break;
        } else {
            name.push(ch as char);
            start += 1;
        }
    }
    name
}
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let start = ptr as usize;
    page_table.translated_va(start.into()).unwrap().get_mut()
}

pub fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T{
    let page_table = PageTable::from_token(token);
    let start = ptr as usize;
    page_table.translated_va(start.into()).unwrap().get_ref()
}

pub struct UserBuffer {
    pub buffer: Vec<&'static mut [u8]>,
}
impl UserBuffer {
    pub fn new(buffer: Vec<&'static mut [u8]>) -> Self {
        Self { buffer }
    }
    pub fn len(&self) -> usize {
        let mut length = 0;
        for buf in self.buffer.iter() {
            length += buf.len();
        }
        length
    }
}
pub struct UserBufferItertor {
    buffer: Vec<&'static mut [u8]>,
    current: usize,
    index: usize,
}
impl Iterator for UserBufferItertor {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.buffer.len() {
            None
        } else {
            let val = &mut self.buffer[self.current][self.index] as *mut u8;
            if self.index + 1 >= self.buffer[self.current].len() {
                self.current += 1;
            } else {
                self.index += 1;
            }
            Some(val)
        }
    }
}
impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferItertor;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferItertor {
            buffer: self.buffer,
            current: 0,
            index: 0,
        }
    }
}
