use crate::system_allocator::linked_list::LinkedListAllocator;

const BLOCK_SIZE: &[usize] = &[8, 16, 32, 64, 128, 512, 1024, 2048];

/// definition for a fixed-size node.
pub struct ListNode {
    next: Option<&'static ListNode>,
}
pub struct FixedSizeAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZE.len()],
    fall_back_allocator: LinkedListAllocator,
}

impl FixedSizeAllocator {
    /// create a new fixed-size allocator.
    pub fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        let allocator = FixedSizeAllocator {
            list_heads: [EMPTY; BLOCK_SIZE.len()],
            fall_back_allocator: LinkedListAllocator::new(),
        };
        allocator
    }
    /// initialize the allocator.
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {}
}
