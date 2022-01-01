use core::panic::PanicInfo;
use crate::sys_exit;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(location) = _info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            _info.message().unwrap()
        );
    } else {
        println!("Panicked :{}", _info.message().unwrap())
    }
    loop{}
    // sys_exit()
}
