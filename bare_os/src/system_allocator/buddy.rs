///! buddy分配器
///! 使用bump分配器分配内存然后在buddy分配器中进行分配管理
use crate::system_allocator::common::{align_up, Locked};
// use crate{DEBUG, INFO};
use crate::system_allocator::bump_allocator::BumpAllocator;
use crate::system_allocator::linked_list::LinkedListAllocator;
use crate::DEBUG;
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::max;
use core::fmt::{Debug, Formatter};
use core::mem::{align_of, size_of};
use core::ptr::null_mut;

const MAXLISTS: usize = 32;

/// Node definition
#[derive(Debug)]
#[repr(C)]
pub struct Node {
    pub next: *mut Node,
    pub prev: *mut Node,
}

impl Node {
    pub fn new() -> Self {
        Node {
            next: null_mut(),
            prev: null_mut(),
        }
    }
}

pub struct Buddy {
    free_lists: [*mut Node; MAXLISTS], //每个队列都是按照2的幂进行对齐
    linked_list: Locked<LinkedListAllocator>,
    max_free_index: usize, //record the max free index
}
impl Debug for Buddy {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "freelists: {:?}, max_free_index: {}",
            self.free_lists, self.max_free_index
        )
    }
}
unsafe impl Send for Buddy {}
impl Buddy {
    pub const fn new() -> Self {
        Self {
            free_lists: [null_mut(); MAXLISTS],
            linked_list: Locked::new(LinkedListAllocator::new()),
            max_free_index: 0,
        }
    }
    pub(crate) fn init(&mut self, heap_start: usize, heap_size: usize) {
        // make sure heap_size >= size_of<Node>
        assert!(heap_size >= size_of::<Node>());
        // init the bump allocator
        self.linked_list.lock().init(heap_start, heap_size);
    }
    /// recycle the memory
    /// 尽量与前面的合并而不是后面
    /// 尽量保持序列有序
    fn delete(&mut self, address: usize, size: usize) {
        //make sure the address align with BuddyNode
        assert_eq!(align_up(address, core::mem::align_of::<Node>()), address);
        assert!(size >= core::mem::size_of::<Node>());
        let pow2 = find_last_min_pow2(size);
        DEBUG!("delete:pow2 {}", pow2);
        self.merge(address, pow2);
    }
    fn merge(&mut self, target_list: usize, pow2: usize) {
        //定位到对应的列表中
        let mut merge_list = &mut self.free_lists[pow2];
        //查找是否有可以合并的node
        let mut find = false;
        DEBUG!(
            "target_list 0x{:x} % pow 0x{:x} = {}",
            target_list,
            1 << (pow2 + 1),
            target_list % (1 << (pow2 + 1))
        );
        let equ = if (target_list % (1 << (pow2 + 1))) == 0 {
            target_list + (1 << pow2)
        } else {
            target_list - (1 << pow2)
        };
        DEBUG!("equ 0x{:x}", equ);
        // 这里我们需要记住合并节点的前节点
        while !merge_list.is_null() {
            if *merge_list as usize == equ {
                find = true;
                break;
            }
            unsafe {
                merge_list = &mut (*(*merge_list)).next;
            }
        }
        DEBUG!("merge:find {}", find);
        if find {
            let record_merge = *merge_list;
            unsafe {
                let pre_merge_list = &mut ((*(*merge_list)).prev);
                let next_merge_list = &mut ((*(*merge_list)).next);
                // 合并前后节点
                if !(*pre_merge_list).is_null() {
                    (*(*pre_merge_list)).next = (*next_merge_list);
                }
                if !(*next_merge_list).is_null() {
                    (*(*next_merge_list)).prev = (*pre_merge_list);
                }
            }
            DEBUG!("merge::record_merge_lsit 0x{:x}", record_merge as usize);
            let target_list = if equ < target_list {
                (record_merge) as usize
            } else {
                target_list
            };
            *merge_list = null_mut();
            DEBUG!("merge::Self {:?}", self);

            self.merge(target_list, pow2 + 1);
        } else {
            merge_list = &mut self.free_lists[pow2];
            DEBUG!("merge:merge_list 0x{:x}", *merge_list as usize);
            let mut target_list = target_list as *mut Node;
            unsafe {
                (*target_list).next = *merge_list;
                (*target_list).prev = null_mut();
                if !(*merge_list).is_null() {
                    (*(*merge_list)).prev = target_list;
                }
            }
            self.free_lists[pow2] = target_list;
        }
    }
    fn inner(&mut self, index: usize) -> *mut u8 {
        let mut target_list = self.free_lists[index];
        if !target_list.is_null() {
            let answer = target_list;
            // DEBUG!("inner:answer 0x{:x}", answer as usize);
            //如果后继节点为空，则直接指向null
            //否则，先将后继节点的前驱节点设置为空
            target_list = unsafe {
                if (*target_list).next.is_null() {
                    null_mut()
                } else {
                    (*(*target_list).next).prev = null_mut();
                    (*target_list).next
                }
            };
            self.free_lists[index] = target_list;
            // DEBUG!("inner:answer 0x{:x}", answer as usize);
            return answer as *mut u8;
        }
        return null_mut();
    }
    fn get(&mut self, layout: Layout) -> *mut u8 {
        //make sure the size is power of 2
        //找到Node和请求内存大小的较大者,并对齐到2的幂次
        let size = layout
            .size()
            .max(size_of::<Node>())
            .max(layout.size())
            .next_power_of_two();
        let align = layout.align().max(size);
        //构造合适的layout转递给linked_listAllocator
        let layout = Layout::from_size_align(size, align).unwrap();
        DEBUG!("{:?}", layout);
        //找到对应列表位置
        let index = find_last_min_pow2(size);
        DEBUG!("get:index {}", index);

        let answer = self.inner(index);
        if !answer.is_null() {
            // DEBUG!("get:answer 0x{:x}", answer as usize);
            return answer;
        }

        //如果列表中不含有可以分配的内存
        let is_enough = self.split(index, layout);
        if !is_enough {
            return null_mut();
        }
        DEBUG!("is_enough: {}", is_enough);
        return self.inner(index);
    }
    fn split(&mut self, pow2: usize, layout: Layout) -> bool {
        let mut index = 0;
        // 找到合适的可以分割的位置
        for i in pow2 + 1..=self.max_free_index as usize {
            if !self.free_lists[i].is_null() {
                index = i;
                break;
            }
        }
        DEBUG!("split:index {}", index);
        if index != 0 {
            for i in (pow2 + 1..=index).rev() {
                let target_list = self.free_lists[i];
                unsafe {
                    if !(*self.free_lists[i]).next.is_null() {
                        self.free_lists[i] = (*self.free_lists[i]).next;
                        (*self.free_lists[i]).prev = null_mut();
                    } else {
                        self.free_lists[i] = null_mut();
                    }
                }
                DEBUG!("split:target_list 0x{:x}", target_list as usize);
                let mid = target_list as usize + (1 << (i - 1));
                let mid_list = mid as *mut usize as *mut Node;
                DEBUG!("split:mid_list 0x{:x}", mid_list as usize);
                unsafe {
                    mid_list.write_volatile(Node::new());
                    (*mid_list).next = self.free_lists[i - 1];
                    if !self.free_lists[i - 1].is_null() {
                        (*self.free_lists[i - 1]).prev = mid_list;
                    }
                    (*target_list).next = mid_list;
                    (*mid_list).prev = target_list;
                    self.free_lists[i - 1] = target_list;
                }
            }
            return true;
        } else {
            //尝试从linkedlistAllocator中分配内存
            let mut req = unsafe { self.linked_list.alloc(layout) };
            DEBUG!(
                "split:req 0x{:x} % {} == {} ",
                req as usize,
                layout.size(),
                req as usize % layout.size()
            );
            if req.is_null() {
                return false;
            }
            let mut req = req as *mut Node;
            unsafe {
                req.write_volatile(Node::new()); //写入node
            }
            DEBUG!("get mem from linkedlistAllocator");
            self.free_lists[pow2] = req;
            return true;
        }
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

unsafe impl GlobalAlloc for Locked<Buddy> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // let size = max(
        //     layout.size().next_power_of_two(),
        //     max(layout.align(), size_of::<Node>()),
        // );//获取实际申请的大小

        let answer = self.lock().get(layout);
        return answer;
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout
            .size()
            .max(size_of::<Node>())
            .max(layout.size())
            .next_power_of_two();
        let align = layout.align().max(size);
        //构造合适的layout转递给linked_listAllocator
        let layout = Layout::from_size_align(size, align).unwrap();
        self.lock().delete(ptr as usize, size);
    }
}

// #[test_case]
pub fn test_buddy() {
    let data = [0 as u128; 8].as_mut_ptr();
    let mut buddy = Buddy::new();
    DEBUG!("data_ptr: 0x{:x}", data as usize);
    buddy.init(data as usize, 16 * 8);
    DEBUG!("{:?}", buddy);
    let layout = Layout::from_size_align(32, 32).expect("adjusting size failed!");
    let answer1 = buddy.get(layout);
    DEBUG!("answer1_ptr: 0x{:x}", answer1 as usize);
    DEBUG!("{:?}", buddy);
    let answer2 = buddy.get(layout);
    DEBUG!("answer2_ptr: 0x{:x}", answer2 as usize);
    DEBUG!("{:?}", buddy);
    let layout_16 = Layout::from_size_align(16, 16).expect("adjusting size failed!");
    let answer3 = buddy.get(layout_16);
    DEBUG!("answer3_ptr: 0x{:x}", answer3 as usize);
    DEBUG!("{:?}", buddy);
    let answer4 = buddy.get(layout_16);
    DEBUG!("answer4_ptr: 0x{:x}", answer4 as usize);
    DEBUG!("{:?}", buddy);

    buddy.delete(answer1 as usize, 32);
    DEBUG!("{:?}", buddy);
    buddy.delete(answer2 as usize, 32);
    DEBUG!("{:?}", buddy);

    let anserw5 = buddy.get(layout);
    DEBUG!("answer5_ptr: 0x{:x}", anserw5 as usize);
    DEBUG!("{:?}", buddy);

    let anserw6 = buddy.get(layout);
    DEBUG!("answer6_ptr: 0x{:x}", anserw6 as usize);
    DEBUG!("{:?}", buddy);

    buddy.delete(anserw5 as usize, 32);
    DEBUG!("{:?}", buddy);
    buddy.delete(anserw6 as usize, 32);
    DEBUG!("{:?}", buddy);
    buddy.delete(answer3 as usize, 16);
    DEBUG!("{:?}", buddy);
    buddy.delete(answer4 as usize, 16);
    DEBUG!("{:?}", buddy);

    let answer7 = buddy.get(layout);
    DEBUG!("answer7_ptr: 0x{:x}", answer7 as usize);
    DEBUG!("{:?}", buddy);
    let layout_64 = Layout::from_size_align(64, 64).expect("adjusting size failed!");
    let answer8 = buddy.get(layout_64);
    DEBUG!("answer8_ptr: 0x{:x}", answer8 as usize);
    DEBUG!("{:?}", buddy);
}
pub fn test_alloc_dealloc() {}
