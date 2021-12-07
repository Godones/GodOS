use core::any::Any;
/// 磁盘块设备的接口定义
/// 在此文件系统的使用者来说，需要实现这些接口
/// 这些接口对应于不同的驱动设备

pub trait BlockDevice: Send + Sync + Any{
    fn read_block(&self,block_id:usize,buf:&mut [u8]);//读取块内容
    fn write_block(&self,block_id:usize,buf:& [u8]);//写块
}