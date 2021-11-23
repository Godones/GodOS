use crate::println;
use alloc::vec::Vec;
use lazy_static::lazy_static;
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
    assert!(app_id < num_app); //判断可用的应用程序
    unsafe {
        let content = core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        );
        println!(
            "[kernel] app_data_size: {}",
            core::mem::size_of_val(content)
        );
        content
    }
}

pub fn get_data_by_name(app_name: &str) -> &'static [u8] {
    (0..get_num_app())
        .find(|&x| APP_NAMES[x] == app_name)
        .map(|x| get_app_data(x))
        .unwrap()
}

pub fn show_apps() {
    println!("****APP****");
    for i in APP_NAMES.iter() {
        println!("{}", i);
    }
    println!("***********");
}
lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let app_nums = get_num_app();
        extern  "C"{
            fn _app_name();
        }
        //起始地址
        let mut start = _app_name as usize as *const u8;
        let mut name = Vec::new();
        unsafe {
            for i in 0..app_nums{
                let mut end = start;
                while end.read_volatile() !='\0' as u8{
                    end = end.add(1);//地址加1
                }//获得每个应用名字
                let slice = core::slice::from_raw_parts(start,end as usize-start as usize);
                let str_name = core::str::from_utf8(slice).unwrap();
                name.push(str_name);
                start = end.add(1);
            }
        }
        name
    };
}
