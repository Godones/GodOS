pub mod address;
pub mod frame_allocator;
pub mod memory_set;
pub mod page_table;

pub use memory_set::KERNEL_SPACE;
use crate::println;
pub fn init() {
    frame_allocator::init_frame_allocator();
    println!("[kernel] init_frame_allocator ok!");
    KERNEL_SPACE.lock().activate();
}
