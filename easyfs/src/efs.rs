use std::sync::Arc;
use crate::bitmap::Bitmap;
use crate::block_dev::BlockDevice;
use spin::Mutex;
///! 负责组织下层抽象的各个文件结构，将其合理安排在磁盘上

pub struct FileSystem {
    pub block_device:Arc<dyn BlockDevice>,
    pub inode_bitmap:Bitmap,
    pub data_bitmap:Bitmap,
    inode_area_blocks: u32,
    data_area_blocks: u32,
}

impl FileSystem {
    pub fn create(
        device:Arc<dyn BlockDevice>,
        total_blocks:usize,
        inode_bitmap_blocks:usize,//索引节点块个数
    ) ->Arc<Mutex<Self>>{
        todo!()
    }
}