///! 文件目录的实现
/// 每一个文件目录的数据结构是一个个目录项的集合
use crate::NAME_LENGTH_MAX;
#[repr(C)]
pub struct DirEntry {
    //文件目录项
    name:[u8;NAME_LENGTH_MAX+1],//文件名/目录名
    node_number:u32,//所在的索引节点编号
}
pub const DIRENTRY_SIZE: usize = 32;

impl DirEntry {
    pub fn empty() -> DirEntry {
        Self{
            name:[0;NAME_LENGTH_MAX+1],
            node_number:0,
        }
    }
    pub fn new(name:&str,node_number:u32)->Self{
        let mut name_byte = [0u8;NAME_LENGTH_MAX+1];
        name_byte[..name.len()].copy_from_slice(name.as_bytes());
        Self{
            name:name_byte,
            node_number,
        }
    }
    pub fn as_bytes(&self) -> &[u8]{
        //将目录项转换位一个缓冲区,使其符合node的read/write
        unsafe{
            core::slice::from_raw_parts(
                self as *const _ as usize as *const u8,
                DIRENTRY_SIZE,
            )
        }
    }
    pub fn as_mut_bytes(&mut self) -> &mut [u8]{
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut _ as usize as *mut u8,
                DIRENTRY_SIZE
            )
        }
    }
    pub fn name(&self)->&str{
        let end = (0usize..).find(|i|self.name[*i]==0).unwrap();
        core::str::from_utf8(&self.name[..end]).unwrap()
    }
    pub fn node_number(&self)->u32{
        self.node_number
    }
}