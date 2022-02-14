mod virtio_block_dev;

use alloc::sync::Arc;
use easyfs::BlockDevice;
use lazy_static::lazy_static;
#[cfg(feature = "board_qemu")]
type BlockDeviceImpl = virtio_block_dev::VirtIOBlock;

#[cfg(feature = "board_k210")]
type BlockDeviceImpl = sdcard::SDCardWrapper;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}
