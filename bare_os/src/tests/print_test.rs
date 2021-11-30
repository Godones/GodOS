use crate::{DEBUG, ERROR, INFO, println, TRACE, WARN};
pub fn color_output_test() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack();
        fn boot_stack_top();
    }
    // clear_bss();
    ERROR!("Hello, world!");
    WARN!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    INFO!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    DEBUG!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    TRACE!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
}
