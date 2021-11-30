pub mod address;
pub mod frame_allocator;
mod memory_set;
pub mod page_table;

use crate::println;
pub use memory_set::{
    remap_test,
    KERNEL_SPACE,
    MapPermission,
    MapArea,
    MapType,
    MemorySet
};

pub fn init() {
    frame_allocator::init_frame_allocator();
    frame_allocator::frame_test();
    println!("[kernel] init_frame_allocator ok!");
    KERNEL_SPACE.lock().activate();
}
