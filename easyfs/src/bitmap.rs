//索引节点区与数据区都需要位图
//这里将位图抽象出来
use crate::block_cache::get_block_cache;
use crate::block_dev::BlockDevice;
use crate::BLOCK_SIZE;
use alloc::sync::Arc;

//位图信息存放于内存中
pub struct Bitmap {
    start_block: usize, //起始块号
    blocks: usize,      //占用数量
}
type BitmapBlock = [u64; 64]; //将一个块的4096 = 512*8表示为u64数组
const BLOCK_BITS: usize = BLOCK_SIZE * 8;

impl Bitmap {
    pub fn new(start_block: usize, blocks: usize) -> Self {
        Self {
            start_block,
            blocks,
        }
    }
    //分配一个位
    pub fn malloc(&mut self, block_device: Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            //查找已有的块中是否还有剩余位置
            let position = get_block_cache(block_id + self.start_block, block_device.clone())
                .lock()
                .modify(0, |bitmap_block: &mut BitmapBlock|{
                    if let Some((bits_pos, inner_pos)) = bitmap_block
                        .iter()
                        .enumerate()
                        .find(|(_, bits64)| **bits64 != u64::MAX)//确认并没有达到最大值
                        .map(|(bits_pos, bits64)| {
                            (bits_pos, bits64.trailing_zeros() as usize)
                        })
                    {
                        bitmap_block[bits_pos] |= 1u64 << inner_pos;
                        Some(block_id * BLOCK_BITS + bits_pos * 64 + inner_pos)
                    } else {
                        None
                    }
                });
            if position.is_some() {
                return position;
            }
        }
        None
    }
    fn depositions(&self, position:usize)->(usize,usize,usize) {
        let block_id = position/BLOCK_SIZE;
        let bits = position%BLOCK_SIZE;
        let bits_pos = bits/64;
        let inner_pos=  bits%64;
        (block_id,bits_pos,inner_pos)
    }
    //回收分配出去的一个位
    pub fn dealloc(&mut self,position:usize,device:Arc<dyn BlockDevice>){
        let (block_id,bits_pos,inner_pos) = self.depositions(position);
        get_block_cache(block_id+self.start_block,device.clone())
            .lock()
            .modify(0,|bitsmap_block:&mut BitmapBlock|{
                //判断当前这个位置是否为1
                assert!(bitsmap_block[bits_pos]&(1u64<<inner_pos)>0);//==1
                bitsmap_block[bits_pos] -= 1u64<<inner_pos;
            });

    }
    pub fn bit_num(&self) -> usize {
        self.blocks * BLOCK_SIZE
    }
}
