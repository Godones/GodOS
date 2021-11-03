///将应用程序全部加载到内存中

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize; //取地址
    unsafe { num_app_ptr.read_volatile() } //读内容 应用数量
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    // 记载各个app到指定的位置，固定分区
    // link_apps按8字节对其
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize; //取地址
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}
