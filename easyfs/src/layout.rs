//描述磁盘的数据结构
//磁盘布局
// 超级快：1块
// 索引节点位图：多块 记录索引节点区域中哪些块被使用
// 索引节点区域：多块 存放所有的索引节点
// 数据块位图： 多块 记录数据库的使用情况
// 数据块：多块 存放所有文件或目录的数据

use crate::{EFS_MAGIC};

#[repr(C)]
pub struct SuperBlock {
    magic: u32,                   //存放魔数，用于验证文件系统的合法性
    pub total_blocks: u32,        //文件系统所占的磁盘块数
    pub inode_bitmap_blocks: u32, //索引节点区域所占块数
    pub inode_area_blocks: u32,   //
    pub data_bitmap_blocks: u32,  //
    pub data_area_blocks: u32,    //
}

impl SuperBlock {
    //初始化超级快
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        *self = Self {
            magic: EFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        };
    }
    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}
