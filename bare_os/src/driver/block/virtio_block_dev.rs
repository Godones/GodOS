use virtio_drivers::{VirtIOBlk,VirtIOHeader};
use spin::Mutex;

const VIRTIO0:usize = 0x10001000;

pub struct VirtIOBlock(Mutex<VirtIOBlk<'static>>);

impl VirtIOBlock{
    pub fn new()->Self{
        Self(Mutex::new(VirtIOBlk::new(
            unsafe {&mut *(VIRTIO0 as *mut VirtIOHeader)},
        )))
    }
}