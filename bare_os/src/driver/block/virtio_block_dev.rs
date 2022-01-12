
use alloc::vec::Vec;
use virtio_drivers::{VirtIOBlk, VirtIOHeader};
use spin::Mutex;
use easyfs::BlockDevice;
use lazy_static::lazy_static;
use crate::mm::address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr};
use crate::mm::frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
use crate::mm::KERNEL_SPACE;
use crate::mm::page_table::PageTable;


const VIRTIO0:usize = 0x10001000;
//虚拟块设备
pub struct VirtIOBlock(Mutex<VirtIOBlk<'static>>);

impl VirtIOBlock{
    pub fn new()->Self{
        //VirtIOHeader 表示以MMIO内存映射方式访问IO设备
        //所需要的一组寄存器
        Self(Mutex::new(VirtIOBlk::new(
            unsafe {&mut *(VIRTIO0 as *mut VirtIOHeader)}
        ).unwrap()))
    }
}

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0.lock().read_block(block_id,buf).expect("Read from block error!");
    }
    //为块设备实现定义的接口
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0.lock().write_block(block_id,buf).expect("Write to block error!");
    }
}
lazy_static!{
    static ref QUEUE_FRAMES:Mutex<Vec<FrameTracker>> = Mutex::new(Vec::new());
}

//为驱动设备实现其定义的接口
//这些接口负责为设备在内存中开辟部分空间用于cpu与设备进行通信
#[no_mangle]
pub extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr{
    //为其分配连续的物理页帧，由于是在内核初始化阶段进行，因此页帧的分配是连续的
    let mut start_ppn = PhysPageNum(0);
    for i in 0..pages{
        let frame = frame_alloc().unwrap();
        if i==0 {
            start_ppn = frame.ppn;
        }
        //判断页帧是否连续
        assert_eq!(frame.ppn.0,start_ppn.0+i);
        QUEUE_FRAMES.lock().push(frame);
    }
    start_ppn.into()
}
#[no_mangle]
pub fn virtio_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32{
    //回收物理页帧
    let mut phypage:PhysPageNum = paddr.into();
    for _ in 0..pages {
        frame_dealloc(phypage);
        phypage.step();
    }
    0
}
#[no_mangle]
pub  fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr{
    //将物理地址转为虚拟地址
    VirtAddr(paddr.0)
}
#[no_mangle]
pub fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr{
    //?为什么这里需要查找页表转换呢？不是恒等映射吗
    PageTable::from_token(KERNEL_SPACE.lock().token()).translated_va(vaddr).unwrap()
}