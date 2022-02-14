///! buddy分配器
use crate::system_allocator::common::{align_up, Locked};
use core::alloc::Layout;
use core::mem::{align_of, size_of};
use core::ptr::null_mut;
use crate::{DEBUG, INFO};

/// Node definition
#[derive(Debug)]
pub struct Node {
    pub next: *mut Node,
}

impl Node {
    pub fn new() -> Self {
        Node { next: null_mut() }
    }
}
const MAXLISTS :usize = 32;
#[derive(Debug)]
pub struct Buddy {
    free_lists:[*mut Node;MAXLISTS],
    heap_start: usize,
    heap_size: usize,
    align_pow2:u8,
}

impl Buddy {
    fn new() -> Self {
        Self {
            free_lists:[null_mut();MAXLISTS],
            heap_start: 0,
            heap_size: 0,
            align_pow2: 0
        }
    }
    fn init(&mut self, heap_start: usize, heap_size: usize) {
        assert!(heap_size>=size_of::<Node>());//heap_size >= 8bytes
        assert_eq!(align_up(heap_start, size_of::<Node>()), heap_start);//heap_start must be align 4096
        self.heap_start = heap_start;
        self.heap_size = heap_size;
        // find the heap_size that can is smaller than pow2
        let min_pow2 = find_last_min_pow2(heap_size);
        assert!(min_pow2 < MAXLISTS);
        self.align_pow2 = min_pow2 as u8;
        //calculate the align_size is pow2
        let max_node = heap_start as *mut Node;
        unsafe {
           // *max_node = Node::new();
            max_node.write_volatile(Node::new());
        }
        self.free_lists[min_pow2] = max_node;
    }
    /// recycle the memory
    /// 尽量与前面的合并而不是后面
    /// 尽量保持序列有序
    fn insert(&self, address: usize, size: usize) {
        assert_eq!(
            align_up(address, core::mem::align_of::<Node>()),
            address
        ); //make sure the address align with BuddyNode
        assert!(size >= core::mem::size_of::<Node>());
        assert!(size <= (1<<self.align_pow2) as usize);
        let size = size.next_power_of_two();
        let pow2 = find_last_min_pow2(size);

    }
    fn get(&mut self, size: usize) ->*mut u8{
        //make sure the size is power of 2
        assert!(size>=core::mem::size_of::<Node>());
        let pow2 = find_last_min_pow2(size.next_power_of_two());
        assert!(pow2 <= self.align_pow2 as usize);
        let mut target_list = self.free_lists[pow2];
        if !target_list.is_null() {
            target_list = unsafe { (*target_list).next };
            return target_list as *mut u8;
        }

        let is_enough = self.split(pow2);
        if  !is_enough{ return null_mut() ;}
        let mut target_list = self.free_lists[pow2];
        if !target_list.is_null() {
            target_list = unsafe { (*target_list).next };
            return target_list as *mut u8;
        }
        null_mut()
    }
    fn split(&mut self, pow2: usize)->bool {
        let mut index = 0;
        // 找到合适的可以分割的位置
        for i in pow2+1..=self.align_pow2 as usize{
           if !self.free_lists[i].is_null(){
               index = i;
               break;
           }
        }
        if index!=0{
            for i in (pow2+1..=index).rev() {
                let target_list = self.free_lists[i];
                unsafe {
                    self.free_lists[i] = (*self.free_lists[i]).next;
                }
                let mid = target_list as usize + (1<<(i-1));
                let mid_list = mid as * mut usize as * mut Node;
                unsafe {
                    mid_list.write_volatile(Node::new());
                    (*mid_list).next = self.free_lists[i-1];
                    (*target_list).next = mid_list;

                    self.free_lists[i-1] = target_list;
                }
            }
            return true
        }
        false
    }
}
/// calculate the last min pow2
pub fn find_last_min_pow2(mut addr: usize) -> usize {
    let mut k = 0;
    while addr > 1 {
        k += 1;
        addr >>= 1;
    }
    return k;
}

pub fn test_buddy(){
    let data = [0 as usize;8].as_mut_ptr();
    let mut buddy = Buddy::new();
    buddy.init(data as usize,64);
    DEBUG!("data_ptr: 0x{:x}",data as usize);
    DEBUG!("{:?}",buddy);
    let answer = buddy.get(8);
    DEBUG!("answer_ptr: 0x{:x}",answer as usize);
    DEBUG!("{:?}",buddy);
}
