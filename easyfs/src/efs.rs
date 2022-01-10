use alloc::sync::Arc;
use crate::bitmap::Bitmap;
use crate::block_dev::BlockDevice;
use spin::Mutex;
use crate::block_cache::get_block_cache;
use crate::BLOCK_SIZE;
use crate::disknode::{DiskNode, DiskNodeType};
use crate::layout::SuperBlock;
use crate::vfs::Inode;

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
        //从第1块开始为索引节点位图所占的块
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
            inode_bitmap,
            data_bitmap,
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

    pub fn open(device:Arc<dyn BlockDevice>)->Arc<Mutex<FileSystem>>{
        //从一个已经写入文件系统的设备上恢复文件系统
        //只要读取第一个存储块将超级快信息读出即可
        get_block_cache(0,device.clone())
            .lock()
            .read(0,|superblock:&SuperBlock|{
                assert!(superblock.is_valid(),"Load error efs");
                let inode_total_blocks = superblock.inode_bitmap_blocks+superblock.inode_area_blocks;
                let efs = Self{
                    block_device:device,
                    inode_bitmap:Bitmap::new(1, superblock.inode_bitmap_blocks as usize),
                    data_bitmap:Bitmap::new((1 + inode_total_blocks) as usize, superblock.data_bitmap_blocks as usize),
                    inode_area_blocks: (1 + superblock.inode_bitmap_blocks ) as u32,
                    data_area_blocks: 1+inode_total_blocks+superblock.data_bitmap_blocks,
                };
                Arc::new(Mutex::new(efs))
            })
    }
    pub fn get_data_block_id(&self,data_block_id:u32)->u32{
       //获取逻辑块号对于物理块号
       self.data_area_blocks + data_block_id
   }
    pub fn alloc_inode(&mut self) ->u32{
        //从索引位图中分配一个inode;
        self.inode_bitmap.alloc(self.block_device.clone()).unwrap() as u32
    }
    pub fn get_disk_inode_pos(&self,inode:u32) -> (u32,usize){
        //根据索引节点号找到位于索引节点区的位置和索引块编号
        let inode_size = inode as usize * core::mem::size_of::<DiskNode>();
        let block_id = inode_size/BLOCK_SIZE + self.inode_area_blocks as usize;
        let offset = inode_size%BLOCK_SIZE;
        (block_id as u32,offset)
    }
    ///将索引块号与块内偏移转化为inode编号
    pub fn get_disk_inode(&self,block_id:u32,offset:usize)->u32{
        let per_block_disknode = BLOCK_SIZE/core::mem::size_of::<DiskNode>();
        let inode = (block_id - self.inode_area_blocks)*per_block_disknode as u32 + offset as u32;
        inode
    }
    pub fn dealloc_data(&mut self,block_id: u32){
        //回收一个数据块
        get_block_cache(block_id as usize,self.block_device.clone())
            .lock()
            .modify(0,|data_block:& mut DataBlock|{
                data_block.iter_mut().for_each(|p| *p =0);
            });//将数据块初始化为0
        // println!("block_id-data_area_blocks = {}",block_id-self.data_area_blocks);
        self.data_bitmap.dealloc((block_id - self.data_area_blocks) as usize, self.block_device.clone());
    }
    pub fn alloc_data(&mut self)->u32{
        self.data_bitmap.alloc(self.block_device.clone()).unwrap() as u32 + self.data_area_blocks
    }
    pub fn root_inode(fs:&Arc<Mutex<FileSystem>>)->Inode{
        //提供根目录的索引节点号
        //其它所有文件或目录均要从根目录开始寻找
        let device = Arc::clone(&fs.lock().block_device);//获取文件系统的块设备引用
        //获取索引节点号为1的索引块位置和偏移量
        let (root_inode_block_id,root_inode_offset) = fs.lock().get_disk_inode_pos(0);
        Inode::new(
            root_inode_block_id,
            root_inode_offset,
            fs.clone(),
            device
        )
    }
}