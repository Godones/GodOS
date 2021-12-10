use alloc::sync::Arc;
use crate::block_dev::BlockDevice;
use crate::{DIRECT_MAX, INDIRECT1_MAX, BLOCK_SIZE, BLOCK_U32, INDIRECT2_MAX};
use crate::block_cache::get_block_cache;

#[derive(PartialEq)]
pub enum  DiskNodeType {
    FILE,
    DIRECTORY,
}

#[repr(C)]
pub struct DiskNode {
    pub size:u32,//记录文件/目录大小
    pub direct:[u32;DIRECT_MAX],//存放数据的块号
    pub indirect1:u32,//一级索引
    pub indirect2:u32,//二级索引
    node_type:DiskNodeType,//文件/目录
}//每个索引节点占据128B
type Indirect = [u32;BLOCK_SIZE/4];//128个u32数据


impl DiskNode {
    pub fn initialize(&mut self,node_type:DiskNodeType){
        self.size = 0;
        self.node_type = node_type;
        self.direct = [0;DIRECT_MAX];
        self.indirect2 = 0;
        self.indirect1 = 0;
    }
    pub fn is_dir(&self)->bool{
        self.node_type==DiskNodeType::DIRECTORY
    }
    pub fn is_file(&self)->bool{
        self.node_type == DiskNodeType::FILE
    }

    pub fn get_block_id(&self,inner_pos:u32,block_device:Arc<dyn BlockDevice>) ->u32{
        //根据inner_pos找到对应的块，inner_pos表示存放数据的某个块号l
        let inner_pos=  inner_pos as usize;
        if inner_pos  < DIRECT_MAX {
            self.direct[inner_pos]//直接索引
        }
        else if inner_pos  < INDIRECT1_MAX{
            //二级索引
            //二级索引块上存放的全部是u32数据类型，指向一个块号
            get_block_cache(self.indirect1 as usize,block_device.clone())
                .lock()
                .read(0,|array:&Indirect|{
                    array[inner_pos-DIRECT_MAX]
                })
        }
        else {
            //三级索引
            //需要计算其所在区间
            let inner_inner_pos = inner_pos - INDIRECT1_MAX;

            let array_index = get_block_cache(self.indirect2 as usize,block_device.clone())
                .lock()
                .read(0,|array:&Indirect|{
                   array[inner_inner_pos/BLOCK_U32]//
                });
            get_block_cache(array_index as usize,block_device.clone())
                .lock()
                .read(0,|array:&Indirect|{
                    array[inner_inner_pos%BLOCK_U32]
                })
        }
    }
}

impl DiskNode {
    ///这部分函数用来给文件扩增数据的时候使用
    fn _data_blocks(size:u32)->u32{
        //向上取整返回这些数据所占用块数量
        (size + BLOCK_SIZE as u32 -1)/(BLOCK_SIZE as u32)
    }
    pub fn data_blocks(&self)->u32{
        Self::_data_blocks(self.size)
    }
    fn total_blocks(&self,size:u32)->u32{
        let data_blocks = Self::_data_blocks(size);
        let mut total_blocks = data_blocks;
        if data_blocks as usize > DIRECT_MAX { total_blocks +=1 };
        if data_blocks as usize > INDIRECT1_MAX{
            total +=1;//二级索引表块
            total_blocks += ((data_blocks - INDIRECT1_MAX as u32)+BLOCK_U32 as u32-1)/(BLOCK_U32 as u32)
        }
        total_blocks
    }
    fn addition_blocks(&self,size:u32)->u32{
        self.total_blocks(size) - self.total_blocks(self.size)
    }
    pub fn increase_size(
        &mut self,
        new_size:u32,//扩充之后文件大小
        device:Arc<dyn BlockDevice>,
        new_blocks:Vec<u32>//装数据的那些块的块号，由上层分配
    ){
        let mut current_data_blocks = self.data_blocks();//当前时刻的数据块数目
        self.size  = new_size;
        let mut after_data_blocks = self.data_blocks();//加入新的数据后的数目
        let mut new_blocks_iter = new_blocks.into_iter();

        while current_data_blocks < after_data_blocks.min(DIRECT_MAX as u32) {
            self.direct[current_data_blocks] = new_blocks_iter.next().unwrap();
            current_data_blocks +=1;//先填满可直接索引区域
        }
        if after_data_blocks > DIRECT_MAX as u32{
            //填充一级索引区域
            if current_data_blocks == DIRECT_MAX as u32{
                //新建一个索引块
                self.indirect1 = new_blocks_iter.next().unwrap();
            }
            current_data_blocks -= DIRECT_MAX;
            after_data_blocks -=DIRECT_MAX;
        }else {
            return //如果加入文件后还是小于直接索引那么就说明前面
        }
        //
        get_block_cache(self.indirect1 as usize, device.clone())
            .lock()
            .modify(0,|array:&Indirect|{
                while current_data_blocks < after_data_blocks.min((INDIRECT1_MAX - DIRECT_MAX) as u32) {
                    array[current_data_blocks] = new_blocks_iter.next().unwrap();
                }
            });
        if after_data_blocks { }

    }
}