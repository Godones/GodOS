#![allow(dead_code)]
use easyfs::{BlockDevice, FileSystem, Inode};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
extern crate clap;
use clap::{App, Arg};

const BLOCK_SIZE: usize = 512;

///使用本地的文件模拟一个块设备
struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
            .expect("Error seeking");
        assert_eq!(
            file.write(buf).unwrap(),
            BLOCK_SIZE,
            "Not a completed block"
        );
    } //通过Seek访问特定的块

    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
            .expect("Error seeking");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SIZE, "Not a completed block");
    } //通过Seek访问特定的块
}

fn crate_filesystem() -> Inode {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("../user/target/riscv64gc-unknown-none-elf/release/fs.img")
            .unwrap();
        f.set_len(8192 * 512 * 4).unwrap(); //设置文件大小
        f
    })));
    //创建文件系统
    FileSystem::create(block_file.clone(), 8192 * 4, 1);
    let fs = FileSystem::open(block_file.clone());
    //得到根目录节点
    let root_inode = FileSystem::root_inode(&fs);
    root_inode
}

#[test]
fn fs_test() -> std::io::Result<()> {
    //测试文件系统
    let root_inode = crate_filesystem();
    root_inode.create("file1");
    root_inode.create("file2");
    println!("test list file names...");
    for l in root_inode.ls() {
        println!("name: {}", l);
    }
    //测试文件写入与读出
    let file1 = root_inode.find_inode("file1").unwrap();
    let hello_str = "Hello world";
    file1.write_at(0, hello_str.as_bytes());
    let mut buffer = [0u8; 255];
    let len = file1.read_at(0, &mut buffer);
    assert_eq!(hello_str, core::str::from_utf8(&buffer[..len]).unwrap());
    println!(
        "{}=={}",
        hello_str,
        core::str::from_utf8(&buffer[..len]).unwrap()
    );

    let file2 = root_inode.find_inode("file2").unwrap();
    let hello_str = "Hello world file2";
    file2.write_at(0, hello_str.as_bytes());
    let mut buffer = [0u8; 255];
    let len = file2.read_at(0, &mut buffer);
    assert_eq!(hello_str, core::str::from_utf8(&buffer[..len]).unwrap());
    println!(
        "{}=={}",
        hello_str,
        core::str::from_utf8(&buffer[..len]).unwrap()
    );

    //测试写入不同长度的内容
    let mut random_str_test = |len: usize| {
        //
        file1.clear(); //清空文件，回收各个数据块
        assert_eq!(file1.read_at(0, &mut buffer), 0); //内容长度为0
        let mut str = String::new();
        for _ in 0..len {
            //随机产生数字
            str.push(char::from('0' as u8 + rand::random::<u8>() % 10));
        }
        println!("str.len():{}", str.len());
        file1.write_at(0, str.as_bytes());

        let mut read_buffer = [0u8; 127];
        let mut offset = 0usize;
        let mut read_str = String::new();
        loop {
            let len = file1.read_at(offset, &mut read_buffer);
            if len == 0 {
                break;
            }
            offset += len;
            read_str.push_str(core::str::from_utf8(&read_buffer[0..len]).unwrap());
        }
        assert_eq!(str, read_str);
    };

    random_str_test(4 * BLOCK_SIZE);
    random_str_test(8 * BLOCK_SIZE + BLOCK_SIZE / 2);
    random_str_test(100 * BLOCK_SIZE);
    random_str_test(70 * BLOCK_SIZE + BLOCK_SIZE / 7);
    random_str_test((12 + 128) * BLOCK_SIZE);
    random_str_test(400 * BLOCK_SIZE);
    random_str_test(1000 * BLOCK_SIZE);
    random_str_test(2000 * BLOCK_SIZE);
    Ok(())
}
// 打包应用程序
fn package() -> std::io::Result<()> {
    let matches = App::new("Get Application Package")
        .version("1.0")
        .author("God")
        .about("Input the source path and the elf_data path")
        .arg(
            Arg::new("source")
                .short('S')
                .long("source")
                .help("Set the source code")
                .takes_value(true),
        )
        .arg(
            Arg::new("target")
                .short('T')
                .long("target")
                .help("Set the target path")
                .takes_value(true),
        )
        .get_matches();
    let source = matches.value_of("source").unwrap(); //获取源文件目录
    let target = matches.value_of("target").unwrap();
    println!("The source path: {}", source);
    println!("The target path: {}", target);
    //获取源文件下各个应用程序的名称
    let mut filenames: Vec<_> = std::fs::read_dir(source)
        .unwrap()
        .into_iter()
        .map(|direntry| {
            let mut name = direntry.unwrap().file_name().into_string().unwrap();
            name.drain(name.find(".").unwrap()..name.len());
            name
        })
        .collect();

    filenames.sort();
    let root_inode = crate_filesystem(); // for name in filenames{

    let mut size_count = 0;
    let mut size_v = Vec::new();
    filenames.iter().for_each(|name| {
        let mut data: Vec<u8> = Vec::new();
        let mut file = std::fs::File::open(format!("{}{}", target, name)).unwrap();
        file.read_to_end(&mut data).unwrap(); //读取完整的应用
        let new_inode = root_inode.create(name.as_str()).unwrap(); //新建一个文件
        size_v.push(data.len());
        // println!("name: {},size:{}",name,data.len());
        new_inode.write_at(0, data.as_slice());
        size_count += new_inode.get_file_size();
    });
    let mut i = 0;
    filenames.iter().for_each(|name|{
        let inode = root_inode.find_inode(name.as_str()).unwrap();
        let size = inode.get_file_size();
        // println!("{}-{}",size_V[i],size);
        assert_eq!(size,size_v[i]);
        i +=1;
    });


    Ok(())
}

fn main() {
    // println!("Test filesystem...");
    package().unwrap();
    // fs_test();
    // test_link();
    // link_test2();
}
