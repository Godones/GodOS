use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use easyfs::{
    Inode,FileSystem
};
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use crate::file::File;
use crate::mm::page_table::UserBuffer;
use crate::driver::BLOCK_DEVICE;
use crate::{DEBUG, println};

pub struct FNode{
    writeable: bool,
    readable: bool,
    inner:Mutex<FNodeInner>,
}
pub struct FNodeInner{
    inode:Arc<Inode>,
    offset:usize,//每个文件的偏移量
}

impl FNode{
    pub fn new(writeable:bool,readable:bool,inode:Arc<Inode>) -> FNode{
        Self{
            writeable,
            readable,
            inner:Mutex::new(
                FNodeInner{
                    inode,
                    offset:0
                }
            )
        }
    }
    pub fn read_all(&self) ->Vec<u8>{
        //读取所有的数据
        let mut data:Vec<u8>= Vec::new();
        let mut inner = self.inner.lock();
        let mut buffer = [0u8;512];
        loop {
            let size = inner.inode.read_at(inner.offset,&mut buffer);
            if size ==0 { break;}
            inner.offset += size;
            data.extend_from_slice(&buffer[..size]);
        }
        // DEBUG!("data_size: {}", data.len());
        data
    }
    pub fn get_file_size(&self) -> usize {
        //
        let inner = self.inner.lock();
        inner.inode.get_file_size()
    }
}

impl File for FNode {
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut read_size = 0;
        for buffer in buf.buffer.iter_mut(){
            DEBUG!("[kernel] offset:{}", inner.offset);
            let size = inner.inode.read_at(inner.offset,*buffer);
            if size==0 {break ;}
            read_size +=size;
            inner.offset +=size;
        }
        read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.lock();
        let mut write_size = 0;
        for buffer in buf.buffer.iter(){
            let size = inner.inode.write_at(inner.offset,*buffer);
            assert_eq!(size,buffer.len());
            write_size += size;
            inner.offset +=size;
        }
        write_size
    }
}
lazy_static! {
    //根目录
    pub static ref ROOT_INODE:Arc<Inode> ={
        let fs = FileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(FileSystem::root_inode(&fs))
    };
}


//文件标志位
bitflags! {
    pub struct OpenFlags:u32{
        const R = 0;
        const W = 1<<0;
        const RW = 1<<1;
        const C = 1<<9;
        const T = 1<<10;
    }
}
pub fn list_apps() {
    println!("******APP LIST******");
    for name in ROOT_INODE.ls().iter() {
        println!("{}", name);
        // if let Some(node) = open_file(name.as_str(), OpenFlags::R) {
        //     println!("file size: {}", node.get_file_size());
        // }
    }
    println!("********************");
}
impl OpenFlags{
    pub fn read_write(&self)->(bool,bool){
        //返回读写位
        if self.is_empty(){
            (true,false)
        }else if self.contains(OpenFlags::W) {
            (false,true)
        }
        else {
            (true,true)
        }
    }
}
pub fn open_file(name:&str,flag:OpenFlags)->Option<Arc<FNode>>{
    let (readable,writeable) = flag.read_write();
    // println!("open file {}",name);
    if flag.contains(OpenFlags::C){
        if let Some(inode) = ROOT_INODE.find_inode(name){
            //如果找到了存在就需要清空内容
            inode.clear();
            Some(Arc::new(
                FNode::new(
                    writeable,
                    readable,
                    inode,
                )
            ))
        }else {
            //没有找到就新建
            ROOT_INODE.create(name)
                .map(|inode|{
                    Arc::new(FNode::new(
                        writeable,
                        readable,
                        inode,
                    ))
                })
        }
    }
    else {

        ROOT_INODE.find_inode(name).
            map(|inode|{
                if flag.contains(OpenFlags::T) {
                    //如果需要截断
                    inode.clear();
                }
                // DEBUG!("[kernel] first_open_size:{}",inode.get_file_size());
                Arc::new(FNode::new(
                    writeable,
                    readable,
                    inode
                ))
            })
    }
}