use std::sync::Arc;
use crate::bitmap::Bitmap;
use crate::block_dev::BlockDevice;
use spin::Mutex;
use crate::block_cache::get_block_cache;
use crate::BLOCK_SIZE;
use crate::inode::{DiskNode, DiskNodeType};
use crate::layout::SuperBlock;

///! 负责组织下层抽象的各个文件结构，将其合理安排在磁盘上

pub struct FileSystem {
    pub block_device:Arc<dyn BlockDevice>,
    pub inode_bitmap:Bitmap,
    pub data_bitmap:Bitmap,
    inode_area_blocks: u32,
    data_area_blocks: u32,
}
type DataBlock = [u8;BLOCK_SIZE];

impl FileSystem {
    pub fn create(
        device:Arc<dyn BlockDevice>,
        total_blocks:usize,
        inode_bitmap_blocks:usize,//索引节点块个数
    ) ->Arc<Mutex<Self>>{
        //从第1块开始位索引节点位图所占的块
        let inode_bitmap = Bitmap::new(1,inode_bitmap_blocks);
        let inode_num = inode_bitmap.bit_num();//索引节点位图所能容纳的节点数目
        //索引节点所占的索引节点区块数
        let inode_area_blocks = (inode_num*core::mem::size_of::<DiskNode>()+BLOCK_SIZE-1)/BLOCK_SIZE;
        let inode_total_blocks = inode_bitmap_blocks + inode_area_blocks;
        //剩余的属于数据节点位图和数据块
        let data_total_blocks = total_blocks - inode_total_blocks;
        let BLOCK_SIZE_BIT = BLOCK_SIZE*8;
        let data_bitmap_blocks = (data_total_blocks+BLOCK_SIZE_BIT-1)/BLOCK_SIZE_BIT;
        let data_area_blocks = total_blocks-1-inode_total_blocks-data_bitmap_blocks;

        let data_bitmap = Bitmap::new(1+inode_total_blocks,data_bitmap_blocks);
        let mut fs = Self{
            block_device:device.clone(),
            inode_bitmap:inode_bitmap,
            data_bitmap:data_bitmap,
            inode_area_blocks: (1 + inode_bitmap_blocks) as u32,
            data_area_blocks: (1 + inode_total_blocks) as u32
        };
        //清空所有的块
        for index in 0..total_blocks{
            get_block_cache(index,Arc::clone(&device))
                .lock()
                .modify(0,|data_block:&mut DataBlock|{
                    for byte in data_block.iter_mut(){
                        *byte = 0;
                    }
                });
        }
        //初始化超级快
        get_block_cache(0,device.clone())
            .lock()
            .modify(0,|super_block:&mut SuperBlock|{
                super_block.initialize(
                    total_blocks as u32,
                    inode_bitmap_blocks as u32,
                    inode_area_blocks as u32,
                    data_bitmap_blocks as u32,
                    data_area_blocks as u32,
                )
            });
        assert!(fs.alloc_inode()==0);
        //建立根目录
        let (root_inode_block_id,root_inode_offset) = fs.get_disk_inode_pos(0);
        get_block_cache(root_inode_block_id as usize, device.clone())
            .lock()
            .modify(root_inode_offset,|disk_inode:&mut DiskNode|{
                disk_inode.initialize(DiskNodeType::DIRECTORY)
            });
        Arc::new(Mutex::new(fs))
    }
    fn alloc_inode(&mut self) ->u32{
        //从索引位图中分配一个inode;
        self.inode_bitmap.malloc(self.block_device.clone()).unwrap() as u32
    }
    fn get_disk_inode_pos(&self,inode:u32) -> (u32,usize){
        //根据索引节点号找到位于索引节点区的位置和索引块编号
        let inode_size = inode as usize*core::mem::size_of::<DiskNode>();
        let block_id = inode_size/BLOCK_SIZE + self.inode_area_blocks as usize;
        let offset = inode_size%BLOCK_SIZE;
        (block_id as u32,offset)
    }
    pub fn open(device:&Arc<dyn BlockDevice>)->Arc<Mutex<FileSystem>>{
        //从一个已经写入文件系统的设备上恢复文件系统
        todo!()
    }
}