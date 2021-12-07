extern crate alloc;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use crate::block_dev::BlockDevice;
///实现磁盘块的缓存

use crate::config::*;

pub struct BlockCache{
    cache:[u8;BLOCK_SIZE],//快缓存
    block_id:usize,//位于磁盘的块号
    block_device:Arc<dyn BlockDevice>,//所属的块设备
    modified:bool,//是否被修改过
}


impl BlockCache{
    pub fn new(block_id:usize,block_device:Arc<dyn BlockDevice>)->Self{
        //从块设备读取id对于的快内容
        let mut cache = [0 as u8;BLOCK_SIZE];
        block_device.read_block(block_id,cache.as_mut());
        Self{
            cache,
            block_id,
            block_device,
            modified:false,
        }
    }
    
    fn addr_offset(&self,offset:usize)->usize{
        //返回偏移后的地址位置
        &self.cache[offset] as *const _ as usize
    }
    pub fn get_ref<T>(&self,offset:usize)->&T
    where T:Sized{
        let t_size = core::mem::size_of::<T>();
        assert!(t_size+offset < BLOCK_SIZE);
        let begin = self.addr_offset(offset);
        unsafe {
            & *(begin as *const T)
        }
    }
    pub fn get_mut<T>(&self,offset:usize)->&mut T
    where T:Sized{
        let t_size = core::mem::size_of::<T>();
        assert!(t_size+offset < BLOCK_SIZE);
        let begin = self.addr_offset(offset);
        unsafe {
            &mut *(begin as *mut T)
        }
    }


    pub fn read<T,V>(&self,offset:usize,f:impl FnOnce(&T)->V)->V{
        //实现对get_ref的包装
        f(self.get_ref(offset))
    }
    pub fn modify<T,V>(&self,offset:usize,f:impl FnOnce(&T)->V)->V{
        //实现对get_mut的包装
        //这里f是一个闭包函数
        f(self.get_mut(offset))
    }

    pub fn sync(&mut self){
        //同步数据
        if self.modified{
            self.modified = false;
            self.block_device.write_block(self.block_id,&self.cache);
        }
    }
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync();
    }
}

pub struct BlockCacheManager{
    queue:VecDeque<(usize,Arc<Mutex<BlockCache>>)>
}

impl BlockCacheManager {
    fn new()->Self{
        Self{
            queue:VecDeque::new()
        }
    }
    pub fn get_block_cache(&mut self, block_id:usize, block_device:Arc<dyn BlockDevice>) ->Arc<Mutex<BlockCache>>{
        if let Some(pair) = self.queue
            .iter()
            .find(|val|val.0==block_id){
            Arc::clone(&pair.1)
        }else {
            //需要查看是否还有空间
            if self.queue.len()==BLOCK_CACHE_SIZE{
                //内存缓冲区已经满了
                if let Some((idx,_)) = self.queue
                    .iter()
                    .enumerate()
                    .find(|(_,pair)|Arc::strong_count(&pair.1)==1){
                    self.queue.drain(idx..=idx);
                }else {
                    panic!("BlockCache is full");
                }
            }
            let block_cache = Arc::new(Mutex::new(BlockCache::new(block_id,block_device)));
            self.queue.push_back((block_id,Arc::clone(&block_cache)));
            block_cache


        }
    }
}


lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER:Mutex<BlockCacheManager> = Mutex::new(BlockCacheManager::new());
}

pub fn get_block_cache(block_id:usize,block_device:Arc<dyn BlockDevice>)->Arc<Mutex<BlockCache>>{
    BLOCK_CACHE_MANAGER.lock().get_block_cache(block_id,block_device)
}