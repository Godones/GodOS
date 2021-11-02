mod address;
mod page_table;
pub mod FrameAllocator;
mod memory_set;

use memory_set::KERNEL_SPACE;

pub fn init(){
    FrameAllocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
}