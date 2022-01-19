use crate::block_cache::get_block_cache;
use crate::block_dev::BlockDevice;
use crate::{BLOCK_SIZE, BLOCK_U32, DIRECT_MAX, INDIRECT1_MAX, INDIRECT2_MAX};
use alloc::sync::Arc;
use alloc::vec::Vec;
#[derive(PartialEq)]
pub enum DiskNodeType {
    FILE,      //普通文件
    DIRECTORY, //目录
}
#[repr(C)]
pub struct DiskNode {
    pub nlink: u32,                //硬链接数量
    pub size: u32,                 //记录文件/目录大小
    pub direct: [u32; DIRECT_MAX], //存放数据的块号
    pub indirect1: u32,            //一级索引
    pub indirect2: u32,            //二级索引
    node_type: DiskNodeType,       //文件/目录
} //每个索引节点占据128B

type Indirect = [u32; BLOCK_SIZE / 4]; //128个u32数据,用来间接索引

impl DiskNode {
    pub fn initialize(&mut self, node_type: DiskNodeType) {
        self.nlink = 1;
        self.size = 0;
        self.node_type = node_type;
        self.direct = [0; DIRECT_MAX];
        self.indirect2 = 0;
        self.indirect1 = 0;
    }
    pub fn is_dir(&self) -> bool {
        self.node_type == DiskNodeType::DIRECTORY
    }
    pub fn is_file(&self) -> bool {
        self.node_type == DiskNodeType::FILE
    }

    ///找到数据块位置
    pub fn get_block_id(&self, inner_pos: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        //根据inner_pos找到对应的块，inner_pos表示存放数据的某个块号l
        let inner_pos = inner_pos as usize;
        if inner_pos < DIRECT_MAX {
            self.direct[inner_pos] //直接索引
        } else if inner_pos < INDIRECT1_MAX {
            //二级索引
            //二级索引块上存放的全部是u32数据类型，指向一个块号
            get_block_cache(self.indirect1 as usize, block_device.clone())
                .lock()
                .read(0, |array: &Indirect| array[inner_pos - DIRECT_MAX])
        } else {
            //三级索引
            //需要计算其所在区间
            let inner_inner_pos = inner_pos - INDIRECT1_MAX;

            let array_index = get_block_cache(self.indirect2 as usize, block_device.clone())
                .lock()
                .read(0, |array: &Indirect| {
                    array[inner_inner_pos / BLOCK_U32] //
                });
            get_block_cache(array_index as usize, block_device.clone())
                .lock()
                .read(0, |array: &Indirect| array[inner_inner_pos % BLOCK_U32])
        }
    }
}

impl DiskNode {
    ///这部分函数用来给文件扩增数据的时候使用
    fn _data_blocks(size: u32) -> u32 {
        //向上取整返回这些数据所占用块数量
        (size + BLOCK_SIZE as u32 - 1) / (BLOCK_SIZE as u32)
    }
    pub fn data_blocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size);
        let mut total_blocks = data_blocks;
        if data_blocks as usize > DIRECT_MAX {
            total_blocks += 1
        };
        if data_blocks as usize > INDIRECT1_MAX {
            total_blocks += 1; //二级索引表块
            total_blocks +=
                ((data_blocks - INDIRECT1_MAX as u32) + BLOCK_U32 as u32 - 1) / (BLOCK_U32 as u32)
        }
        total_blocks
    }
    pub fn addition_blocks(&self, new_size: u32) -> u32 {
        //求出扩容后需要增加的数据块与索引块
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }
    pub fn increase_size(
        &mut self,
        new_size: u32, //扩充之后文件大小
        device: &Arc<dyn BlockDevice>,
        new_blocks: Vec<u32>, //上层传来的申请的存储块，可以做数据块和索引块
    ) {
        let mut current_data_blocks = self.data_blocks() as usize; //当前时刻的数据块数目
        self.size = new_size; //更改文件/目录大小
        let mut after_data_blocks = self.data_blocks() as usize; //加入新的数据后的数目

        // println!("[filesystem]inode::increase_size::after-current = {}",after_data_blocks-current_data_blocks);
        let mut new_blocks_iter = new_blocks.into_iter();

        while current_data_blocks < after_data_blocks.min(DIRECT_MAX) {
            self.direct[current_data_blocks as usize] = new_blocks_iter.next().unwrap();
            current_data_blocks += 1; //先填满可直接索引区域
        }
        if after_data_blocks > DIRECT_MAX {
            //填充一级索引区域
            if current_data_blocks == DIRECT_MAX {
                //新建一个索引块
                self.indirect1 = new_blocks_iter.next().unwrap();
            }
            current_data_blocks -= DIRECT_MAX;
            after_data_blocks -= DIRECT_MAX;
        } else {
            return; //如果加入文件后还是小于直接索引那么就说明前面已经添加完成
        }
        //获取一级索引的缓存
        //30 - 28 = 2
        //33 - 28 = 5
        get_block_cache(self.indirect1 as usize, device.clone())
            .lock()
            .modify(0, |array: &mut Indirect| {
                while current_data_blocks < after_data_blocks.min(BLOCK_U32) {
                    array[current_data_blocks] = new_blocks_iter.next().unwrap();
                    current_data_blocks += 1;
                }
            });
        if after_data_blocks > (INDIRECT1_MAX - DIRECT_MAX) {
            //如果大于一级索引所能容纳的数量
            if current_data_blocks == (INDIRECT1_MAX - DIRECT_MAX) {
                self.indirect2 = new_blocks_iter.next().unwrap();
            }
            current_data_blocks -= INDIRECT1_MAX - DIRECT_MAX;
            after_data_blocks -= INDIRECT1_MAX - DIRECT_MAX;
        } else {
            return;
        }
        //
        let mut c_first = current_data_blocks / (INDIRECT1_MAX - DIRECT_MAX);
        let mut c_second = current_data_blocks % (INDIRECT1_MAX - DIRECT_MAX);
        let a_first = after_data_blocks / (INDIRECT1_MAX - DIRECT_MAX);
        let a_second = after_data_blocks % (INDIRECT1_MAX - DIRECT_MAX);
        get_block_cache(self.indirect2 as usize, device.clone())
            .lock()
            .modify(0, |array: &mut Indirect| {
                while (c_first < a_first) || (c_first == a_first && c_second < a_second) {
                    //如果第一级的index不等于或者第二级index不等于
                    if c_second == 0 {
                        array[c_first] = new_blocks_iter.next().unwrap();
                    }
                    get_block_cache(array[c_first] as usize, device.clone())
                        .lock()
                        .modify(0, |array1: &mut Indirect| {
                            array1[c_second] = new_blocks_iter.next().unwrap();
                        });
                    c_second += 1;
                    if c_second == BLOCK_U32 {
                        c_first += 1;
                        c_second = 0;
                    }
                }
            });
    }
    pub fn clear_size(&mut self, device: &Arc<dyn BlockDevice>) -> Vec<u32> {
        //清空文件数据后应该回收所有的数据和索引块
        let mut useless_block: Vec<u32> = Vec::new();
        let mut data_blocks = self.data_blocks() as usize;
        self.size = 0; //文件大小为0
        let mut current_blcoks = 0;
        while current_blcoks < data_blocks.min(DIRECT_MAX) {
            useless_block.push(self.direct[current_blcoks]);
            self.direct[current_blcoks] = 0;
            current_blcoks += 1;
        }
        if data_blocks > DIRECT_MAX {
            useless_block.push(self.indirect1);
            current_blcoks = 0;
            data_blocks -= DIRECT_MAX;
        } else {
            return useless_block;
        }
        get_block_cache(self.indirect1 as usize, device.clone())
            .lock()
            .modify(0, |array: &mut Indirect| {
                while current_blcoks < data_blocks.min(BLOCK_U32) {
                    useless_block.push(array[current_blcoks]);
                    current_blcoks += 1;
                }
            });
        self.indirect1 = 0;
        if data_blocks > BLOCK_U32 {
            useless_block.push(self.indirect2);
            data_blocks -= BLOCK_U32;
        } else {
            return useless_block;
        }

        let a1 = data_blocks / BLOCK_U32;
        let b1 = data_blocks % BLOCK_U32;

        get_block_cache(self.indirect2 as usize, device.clone())
            .lock()
            .modify(0, |array1: &mut Indirect| {
                for i in 0..a1 {
                    useless_block.push(array1[i]);
                    get_block_cache(array1[i] as usize, device.clone())
                        .lock()
                        .modify(0, |array2: &mut Indirect| {
                            for j in 0..BLOCK_U32 {
                                useless_block.push(array2[j])
                            }
                        });
                }
                if b1 > 0 {
                    useless_block.push(array1[a1]);
                    get_block_cache(array1[a1] as usize, device.clone())
                        .lock()
                        .modify(0, |array2: &mut Indirect| {
                            for j in 0..b1 {
                                useless_block.push(array2[j])
                            }
                        });
                }
            });

        self.indirect2 = 0;
        useless_block
    }
}

type DataBlock = [u8; BLOCK_SIZE];

impl DiskNode {
    pub fn read_at(&self, offset: usize, buf: &mut [u8], device: &Arc<dyn BlockDevice>) -> usize {
        // println!("{}",self.size);
        let mut start = offset;
        //判断本文件大小
        let end = (offset + buf.len()).min(self.size as usize);
        if start > end {
            return 0;
        }
        let mut start_block = start / BLOCK_SIZE; //起始块
        let mut read_size = 0usize;
        loop {
            //计算本数据块的的末位位置
            let mut current_end_blcok = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            current_end_blcok = current_end_blcok.min(end);
            let current_read_size = current_end_blcok - start;
            //目标缓冲区
            let dst = &mut buf[read_size..read_size + current_read_size];
            get_block_cache(
                self.get_block_id(start_block as u32, device) as usize,
                device.clone(),
            )
            .lock()
            .read(0, |array: &DataBlock| {
                let src = &array[start % BLOCK_SIZE..start % BLOCK_SIZE + current_read_size];
                dst.copy_from_slice(src);
            });
            read_size += current_read_size;
            if current_end_blcok == end {
                break;
            } //读完
            start_block += 1;
            start = current_end_blcok;
        }
        read_size
    }
    pub fn write_at(&self, offset: usize, buf: &[u8], device: &Arc<dyn BlockDevice>) -> usize {
        let mut start = offset;
        //判断本文件大小
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(start <= end);
        let mut start_block = start / BLOCK_SIZE; //起始块
        let mut write_size = 0usize;
        loop {
            //计算本数据块的的末位位置
            let mut current_end_blcok = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            current_end_blcok = current_end_blcok.min(end);
            //计算要写入的大小
            let current_write_size = current_end_blcok - start;

            get_block_cache(
                self.get_block_id(start_block as u32, device) as usize,
                device.clone(),
            )
            .lock()
            .modify(0, |array: &mut DataBlock| {
                //目标缓冲区
                let src = &buf[write_size..write_size + current_write_size];
                let dst = &mut array[start % BLOCK_SIZE..start % BLOCK_SIZE + current_write_size];
                dst.copy_from_slice(src);
            });
            write_size += current_write_size;
            if current_end_blcok == end {
                break;
            } //读完
            start_block += 1;
            start = current_end_blcok;
        }
        write_size
    }
}
