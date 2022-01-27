use crate::block_cache::{block_cache_sync, get_block_cache};
use crate::block_dev::BlockDevice;
use crate::dir_entry::{DirEntry, DIRENTRY_SIZE};
use crate::disknode::{DiskNode, DiskNodeType};
use crate::efs::FileSystem;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use spin::MutexGuard;
///! 索引节点层，负责提供系统调用

///Inode与Disknode的区别在于Inode存在于内存中
///记录文件索引节点的相关信息
/// 可以根据id、offset找到disknode
pub struct Inode {
    block_id: usize,        //物理块号->存放索引节点的块号
    block_offset: usize,    //索引块内偏移
    fs: Arc<Mutex<FileSystem>>, //文件系统指针，各个索引节点均需要通过这个实际操作磁盘
    block_device: Arc<dyn BlockDevice>,
}
impl Inode {
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<FileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }

    pub fn get_disk_nlink(&self) -> u32 {
        self.read_disk_inode(|disknode| disknode.nlink)
    }

    pub fn add_disk_nlink(&self) {
        self.modify_disk_inode(|disknode| {
            disknode.nlink += 1;
        })
    }
    pub fn sub_disk_nlink(&self) {
        self.modify_disk_inode(|disknode| {
            disknode.nlink -= 1;
        })
    }
    ///查看文件大小
    pub fn get_file_size(&self) -> usize {
        self.read_disk_inode(|disk_node| disk_node.size as usize)
    }
    ///查看文件类型
    pub fn get_disk_type(&self) -> u32 {
        self.read_disk_inode(|disknode| {
            if disknode.is_dir() {
                0o040000
            } else {
                0o100000
            }
        })
    }
    ///查看文件inode编号
    pub fn get_disk_inode(&self) -> usize {
        let fs = self.fs.lock();
        fs.get_disk_inode(self.block_id as u32, self.block_offset) as usize
    }

    pub fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskNode) -> V) -> V {
        //读取索引节点的内容
        get_block_cache(self.block_id, self.block_device.clone())
            .lock()
            .read(self.block_offset, f)
    }
    pub fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskNode) -> V) -> V {
        //修改索引节点内容
        get_block_cache(self.block_id, self.block_device.clone())
            .lock()
            .modify(self.block_offset, f)
    }
    pub fn find_inode(&self, name: &str) -> Option<Arc<Inode>> {
        //根据名称找到文件索引节点号
        let fs = self.fs.lock(); //尝试获得文件系统的互斥锁
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Inode::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }
    pub fn find_inode_id(&self, name: &str, disk_inode: &DiskNode) -> Option<u32> {
        //判断是否是目录
        assert!(disk_inode.is_dir(), "This is not a directory");
        let mut direntry = DirEntry::empty(); //创建一个空的目录项
        let direntry_num = disk_inode.size as usize / DIRENTRY_SIZE; //目录文件中包含的目录项数目
        for index in 0..direntry_num {
            assert_eq!(
                disk_inode.read_at(
                    index * DIRENTRY_SIZE,
                    direntry.as_mut_bytes(),
                    &self.block_device
                ),
                DIRENTRY_SIZE
            );
            if direntry.name() == name {
                return Some(direntry.node_number());
            }
        }
        None
    }
    pub fn ls(&self) -> Vec<String> {
        //列举目录下的所有文件名
        let _fs = self.fs.lock(); //防止在多核时其它核抢占文件系统锁
        self.read_disk_inode(|disk_inode| {
            let file_count = disk_inode.size as usize / DIRENTRY_SIZE; //目录项
            let mut file_names: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut direntry = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(
                        i * DIRENTRY_SIZE,
                        direntry.as_mut_bytes(),
                        &self.block_device
                    ),
                    DIRENTRY_SIZE
                ); //判断是否是一个正确的目录项

                file_names.push(String::from(direntry.name()));
            }
            file_names
        })
    }
    pub fn create_nlink(&self, newname: &str, oldname: &str) -> Option<Arc<Inode>> {
        //创建一个硬链接文件
        if self
            .modify_disk_inode(|root_node: &mut DiskNode| {
                //查找是否已经存在此节点
                assert!(root_node.is_dir(), "The root node is not directory");
                self.find_inode_id(newname, root_node) //在根目录下查找
            })
            .is_some()
        {
            return None; //存在文件
        }
        //新建一个文件
        let old_inode = self.find_inode(oldname).unwrap();
        // println!("oldinode:{}-{}",old_inode.block_id,old_inode.block_offset);
        let (inode_block_id, inode_block_offset) = (old_inode.block_id, old_inode.block_offset);
        self.modify_disk_inode(|root_inode| {
            //在根目录下添加
            let file_num = root_inode.size as usize / DIRENTRY_SIZE;
            let new_size = (file_num + 1) * DIRENTRY_SIZE; //新的目录大小
                                                           // println!("!!!!--- {}",old_inode.get_disk_inode());
            let new_entry = DirEntry::new(newname, old_inode.get_disk_inode() as u32);
            let mut fs = self.fs.lock();
            self.increase_size(new_size as u32, root_inode, &mut fs);

            let _number = root_inode.write_at(
                file_num * DIRENTRY_SIZE as usize,
                new_entry.as_bytes(),
                &self.block_device,
            ); //写入目录项
        });
        let new_inode = Inode::new(
            inode_block_id as u32,
            inode_block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        );
        new_inode.add_disk_nlink(); //添加硬链接
        Some(Arc::new(new_inode))
    }
    pub fn delete_nlink(&self, path: &str) -> isize {
        //只需要找到文件inode并且将其换成一个空白文件即可
        let inode = self.find_inode(path).unwrap(); //找到文件inode
                                                    //需要从目录下删除文件并减少文件的硬链接计数
        inode.sub_disk_nlink();
        if inode.get_disk_nlink() == 0 {
            inode.clear();
        }
        0
    }

    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        //创建一个文件/目录，这里只实现创建一个文件
        let mut fs = self.fs.lock();
        if self
            .modify_disk_inode(|root_node: &mut DiskNode| {
                //查找是否已经存在此节点
                assert!(root_node.is_dir(), "The root node is not directory");
                self.find_inode_id(name, root_node) //在根目录下查找
            })
            .is_some()
        {
            return None; //存在文件
        }
        //新建一个文件
        let inode_id = fs.alloc_inode();
        let (inode_block_id, inode_block_offset) = fs.get_disk_inode_pos(inode_id);
        // println!("create {}-{}-{}",inode_id,inode_block_id,inode_block_offset);
        get_block_cache(inode_block_id as usize, self.block_device.clone())
            .lock()
            .modify(inode_block_offset, |new_disk_inode: &mut DiskNode| {
                new_disk_inode.initialize(DiskNodeType::FILE);
            });
        self.modify_disk_inode(|root_inode| {
            //在根目录下添加
            let file_num = root_inode.size as usize / DIRENTRY_SIZE;
            let new_size = (file_num + 1) * DIRENTRY_SIZE; //新的目录大小
            self.increase_size(new_size as u32, root_inode, &mut fs);
            let new_entry = DirEntry::new(name, inode_id);

            let _number = root_inode.write_at(
                file_num * DIRENTRY_SIZE as usize,
                new_entry.as_bytes(),
                &self.block_device,
            ); //写入目录项
               // println!("The filenames: {},the inode_id: {},offset: {}",name,inode_id,file_num*DIRENTRY_SIZE);
        });
        // let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
        block_cache_sync(); //同步数据到磁盘
        Some(Arc::new(Inode::new(
            inode_block_id,
            inode_block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
    }
    pub fn increase_size(
        &self,
        new_size: u32,
        disk_node: &mut DiskNode,
        fs: &mut MutexGuard<FileSystem>,
    ) {
        //增加文件大小
        assert!(new_size > disk_node.size);
        //先计算需要增加的大小,需要添加的索引块和数据块
        let block_need_add = disk_node.addition_blocks(new_size);
        // println!("[filesystem]vfs::increase_size::block_need_add:{}",block_need_add);
        let mut alloc_block_ids: Vec<u32> = Vec::new();
        for _ in 0..block_need_add {
            alloc_block_ids.push(fs.alloc_data()); //分配一些数据块
        }
        disk_node.increase_size(new_size, &self.block_device, alloc_block_ids);
    }
}

impl Inode {
    pub fn clear(&self) {
        //清空文件内容
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_node| {
            let size = disk_node.size;
            let block_realloc_ids = disk_node.clear_size(&self.block_device);
            //判断回收回来的数据块是否正确
            assert_eq!(
                block_realloc_ids.len(),
                DiskNode::total_blocks(size) as usize
            );
            block_realloc_ids
                .into_iter()
                .for_each(|indx| fs.dealloc_data(indx)); //回收数据块，将会清0
        })
    }
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        //对文件进行读写
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_node| disk_node.read_at(offset, buf, &self.block_device))
    }
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        // println!("[filesystem]vfs::write_at");
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_node| {
            //先扩容
            self.increase_size((offset + buf.len()) as u32, disk_node, &mut fs);
            disk_node.write_at(offset, buf, &self.block_device)
        })
    }
}
