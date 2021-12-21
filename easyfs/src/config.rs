pub const BLOCK_SIZE: usize = 512; //块大小
pub const BLOCK_CACHE_SIZE: usize = 16; //在内存驻留的快缓存数量
pub const EFS_MAGIC: u32 = 0x3b800001;//文件系统的标识符
pub const BLOCK_U32:usize = BLOCK_SIZE / 4;
pub const DIRECT_MAX: usize = 28;//一级直接索引
pub const INDIRECT1_MAX:usize = BLOCK_U32+DIRECT_MAX;//二级索引的最大块号
pub const INDIRECT2_MAX: usize = DIRECT_MAX + (BLOCK_U32)^2;//三级索引最大块号
pub const NAME_LENGTH_MAX:usize = 27;