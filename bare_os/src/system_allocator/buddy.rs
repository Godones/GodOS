//buddy分配器
#![allow(unused)]
use crate::system_allocator::common::{align_up, Locked};
use core::alloc::Layout;
pub struct BuddyNode {
    start: usize, //起始地址
    size: usize,  //地址范围大小
    count: usize, //地址范围内的块个数
    left: Option<&'static BuddyNode>,
    right: Option<&'static BuddyNode>,
}

impl BuddyNode {
    fn new(start: usize, size: usize) -> Self {
        Self {
            start,
            size,
            count: 0,
            left: None,
            right: None,
        }
    }
}
pub struct Buddy {
    root: BuddyNode,
}

impl Buddy {
    fn new() -> Self {
        Self {
            root: BuddyNode::new(0, 0),
        }
    }
    fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.root.start = heap_start;
        self.root.size = heap_size;
    }
    fn push(&self, address: usize, size: usize) {
        //回收内存
        assert_eq!(
            align_up(address, core::mem::align_of::<BuddyNode>()),
            address
        );
        assert!(size >= core::mem::size_of::<BuddyNode>());
        assert!(address < self.root.start + self.root.size && address >= self.root.start);
        //地址范围正确
        //todo("递归查找应该合并的节点")
        //根据size和address从树根往下查找，直到找到对应的位置
    }
    fn pop(&self, size: usize, align: usize) {
        //分配内存
        todo!()
    }
}
