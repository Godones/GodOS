use alloc::sync::Arc;
use spin::Mutex;
use crate::block_cache::get_block_cache;
use crate::block_dev::BlockDevice;
use crate::dir_entry::{DirEntry, DIRENTRY_SIZE};
use crate::efs::FileSystem;
use crate::inode::DiskNode;

///! 索引节点层，负责提供系统调用

pub struct Inode{
    //Inode与Disknode的区别在于Inode存在于内存中
    //记录文件索引节点的相关信息
    block_id:usize,//物理块号
    block_offset:usize,//块内偏移
    fs:Arc<Mutex<FileSystem>>,//文件系统指针，各个索引节点均需要通过这个实际操作磁盘
    block_device:Arc<dyn BlockDevice>
}
impl Inode{
    pub fn new(
        block_id:u32,
        block_offset:usize,
        fs:Arc<Mutex<FileSystem>>,
        block_device:Arc<dyn BlockDevice>
    )->Self{
        Self{
            block_id:block_id as usize,
            block_offset,
            fs,
            block_device
        }
    }
    pub fn read_disk_inode<V>(&self,f:impl FnOnce(&DiskNode)->V)->V{
        //读取索引节点的内容
        get_block_cache(self.block_id,self.block_device.clone())
            .lock()
            .read(self.block_offset,f)
    }
    pub fn modify_disk_inode<V>(&self,f:impl FnOnce(&mut DiskNode)->V)->V{
        //修改索引节点内容
        get_block_cache(self.block_id,self.block_device.clone())
            .lock()
            .modify(self.block_offset,f)
    }
    pub fn find_inode(&self,name:&str)->Option<Arc<Inode>>{
        //根据名称找到文件索引节点号
        let fs = self.fs.lock();//尝试获得文件系统的互斥锁
        self.read_disk_inode(|disk_inode|{
            self.find_inode_id(name,disk_inode).map(|inode_id|{
               let(block_id,block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(
                    Inode::new(
                        block_id,
                        block_offset,
                        self.fs.clone(),
                        self.block_device.clone(),
                    )
                )
            })
        })
    }
    fn find_inode_id(&self,name:&str,disk_inode:&DiskNode)->Option<u32>{
        //判断是否是目录
        assert!(disk_inode.is_dir(),"This is not a directory");
        let mut direntry = DirEntry::empty();//创建一个空的目录项
        let direntry_num = disk_inode.size as usize/DIRENTRY_SIZE;//目录文件中包含的目录项数目
        for index in 0..direntry_num{
            assert_eq!(
                disk_inode.read_at(
                    index*DIRENTRY_SIZE,
                    direntry.as_mut_bytes(),
                    &self.block_device
                ),DIRENTRY_SIZE);
                if direntry.name()==name{
                    return Some(direntry.node_number())
                }
        }
        None
    }
    pub fn ls(&self)->Vec<String>{
        //列举目录下的所有文件名
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode|{
            let file_count = disk_inode.size as usize/DIRENTRY_SIZE;//目录项
            let mut file_names :Vec<String>= Vec::new();
            for i in 0..file_count{
                let mut direntry = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(
                        i*DIRENTRY_SIZE,
                        direntry.as_mut_bytes(),
                        &self.block_device
                    ),DIRENTRY_SIZE
                );//判断是否是一个正确的目录项
                file_names.push(String::from(direntry.name()));
            }
            file_names
        })

    }
}