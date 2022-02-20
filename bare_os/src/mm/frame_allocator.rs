///物理页帧分配器
use crate::config::MEMORY_END;
use crate::mm::address::{PhysAddr, PhysPageNum};
use crate::INFO;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::option::Option;
use lazy_static::lazy_static;
use spin::Mutex;

//全局分配器
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<StackFrameAllocator> =
        Mutex::new(StackFrameAllocator::new());
}
extern "C" {
    fn ekernel();
}

pub fn init_frame_allocator() {
    //初始化分配器
    INFO!("[kernel] frame: {}-{}", ekernel as usize, MEMORY_END);
    FRAME_ALLOCATOR.lock().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}
//分配器trait，定义了一个物理页帧分配器应该实现的功能
trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize,       //起始页帧
    end: usize,           //终止页帧
    recycled: Vec<usize>, //回收的页帧
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        //先从回收的页帧中分配出去，若找不到再从未分配的里面分配出去
        // INFO!("[kernel] mm::FrameAllocator::alloc");
        if let Some(page) = self.recycled.pop() {
            Some(page.into())
        } else if self.current < self.end {
                self.current += 1;
                Some((self.current - 1).into())
        } else {
            None
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        //回收页帧
        let ppn: usize = ppn.into();
        //查找分配栈中是否已经存在页和此页相同，若有相同的则出错
        if ppn > self.current || self.recycled.iter().find(|&page| *page == ppn).is_some() {
            panic!("Frame ppn:{:#x} has not been allocated", ppn);
        }
        self.recycled.push(ppn.into());
    }
}

impl StackFrameAllocator {
    //设置页帧起始与结尾
    fn init(&mut self, begin: PhysPageNum, end: PhysPageNum) {
        self.current = begin.into();
        self.end = end.into();
    }
}
#[derive(Debug)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // INFO!("[kernel] mm::FrameAllocator::FrameTracker::new");
        let bytes_array = ppn.get_bytes_array(); //指针数组
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn)
    }
}

///公开的接口
pub fn frame_alloc() -> Option<FrameTracker> {
    //返回一个FrameTracker的原因：
    // 再包装一层需要清零？
    // RAII 的思想，生命周期绑定
    // 为啥不直接再physpagenum上面实现Drop呢
    FRAME_ALLOCATOR
        .lock()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(ppn);
}

#[allow(unused)]
pub fn frame_test() {
    // init_frame_allocator();
    let mut framepages: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let temp = frame_alloc().unwrap();
        // framepages.push(temp);
    }
    framepages.clear();
    for i in 0..5 {
        let temp = frame_alloc().unwrap();
        framepages.push(temp);
    }
    drop(framepages);
    INFO!("Frame_alloc_test passed!");
}
