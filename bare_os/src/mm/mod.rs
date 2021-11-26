pub mod address;
pub mod frame_allocator;
pub mod memory_set;
pub mod page_table;

use crate::println;
pub use memory_set::remap_test;
pub use memory_set::{KERNEL_SPACE,MapPermission};
pub fn init() {
    frame_allocator::init_frame_allocator();
    frame_allocator::frame_test();
    println!("[kernel] init_frame_allocator ok!");
    KERNEL_SPACE.lock().activate();
}
