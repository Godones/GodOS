mod virtio_block_dev;

use lazy_static::lazy_static;
use alloc::sync::Arc;
use easyfs::BlockDevice;
#[cfg(feature = "board_qemu")]
type BlockDeviceImpl = virtio_block_dev::VirtIOBlock;

#[cfg(feature = "board_k210")]
type BlockDeviceImpl = sdcard::SDCardWrapper;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice > = Arc::new(BlockDeviceImpl::new());
}