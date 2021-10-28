use crate::config::KERNEL_HEAP_SIZE;
use crate::system_allocator::bump_allocator::BumpAllocator;
use crate::system_allocator::common::Locked;
use crate::system_allocator::linked_list::LinkedListAllocator;
use crate::INFO;
/// 实现自己的堆分配器
mod bump_allocator;
mod common;
mod buddy;
mod linked_list;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
//
pub fn init_heap() {
    unsafe {
        ALLOCATOR
            .lock()
            .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

#[allow(unused)]
pub fn heap_test() {
    init_heap();
    use alloc::boxed::Box; //使用Box包装器
    use alloc::vec::Vec; //使用vec数组
    extern "C" {
        fn sbss();
        fn ebss();
    }

    let bss_range = sbss as usize..ebss as usize;

    let a = Box::new(5);

    assert_eq!(*a, 5);

    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    //判断指针是否指向bss段
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    INFO!("[kernel] heap_test passed!");
}
