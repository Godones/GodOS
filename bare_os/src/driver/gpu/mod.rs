use lazy_static::lazy_static;
use virtio_drivers::{VirtIOGpu,VirtIOHeader};
use crate::INFO;


mod virtio_gpu;

const VIRTIO0:usize =  0x3000000;
// lazy_static! {
//     pub static ref GPU: Arc<VirtIoGpu> = Arc::new(VirtIoGpu::new());
// }
pub fn gpu() {
    INFO!("set the gpu");
    let mut gpu = VirtIOGpu::new(unsafe {&mut *(VIRTIO0 as *mut VirtIOHeader)})
        .expect("failed to create gpu driver");

    INFO!("GET GPU");
    let fb = gpu.setup_framebuffer()
        .expect("failed to get fb");

    for y in 0..768 {
        for x in 0..1024 {
            let idx = (y * 1024 + x) * 4;
            fb[idx] = (0) as u8;       //Blue
            fb[idx + 1] = (0) as u8;   //Green
            fb[idx + 2] = (255) as u8; //Red
            fb[idx + 3] = (0) as u8;   //Alpha
        }
    }
    gpu.flush().expect("failed to flush");
    INFO!("virtio-gpu test finished");
}