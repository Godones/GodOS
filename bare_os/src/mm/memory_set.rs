use crate::config::{MEMORY_END, PAGE_SIZE, TRAMPOLINE, USER_STACK_SIZE, MMIO};
use crate::mm::address::{PhysAddr, PhysPageNum, StepByOne, VPNRange, VirtAddr, VirtPageNum};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};
use crate::mm::page_table::{PTEFlags, PageTable, PageTableEntry};
use crate::{println, INFO};
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use lazy_static::lazy_static;
use riscv::register;
use spin::Mutex;
use xmas_elf::ElfFile;
use core::arch::asm;
/// 地址空间的抽象
/// 对于任意一个应用程序(后面成为进程）来说，其由多个
/// 段构成，每个段对应于一段虚拟的逻辑地址空间
/// 而管理应用程序就是管理一系列逻辑段
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MapType {
    Identical, //恒等映射==>logical address = physaddress
    Framed,    //其它映射
}
bitflags! {
    pub struct MapPermission:u8{
        const R = 1<<1;
        const W = 1<<2;
        const X = 1<<3;
        const U = 1<<4;
    }
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline(); //跳板位置？
}

pub struct MapArea {
    //一段逻辑地址空间的描述
    vpn_range: VPNRange, //虚拟页号的迭代器
    //虚拟页号和物理页号的对应关系
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    //逻辑段的映射方式
    map_type: MapType,
    //逻辑段的读取权限
    map_perm: MapPermission,
}

pub struct MemorySet {
    //应用程序的地址空间
    //三级页表
    page_table: PageTable,
    //所有的逻辑段
    areas: Vec<MapArea>,
}

impl MemorySet {
    fn new_bare() -> Self {
        //空的地址空间
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    pub fn activate(&self) {
        //激活虚拟页表功能
        let satp = self.page_table.token();
        unsafe {
            register::satp::write(satp);
            asm!("sfence.vma", options(nostack))
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        //插入一个段,并可以在映射的物理页帧上写入数据
        //map方法会 在页表中添加这个段对应的虚拟页号和物理页号
        map_area.map(&mut self.page_table);
        if let Some(value) = data {
            map_area.copy_data(&mut self.page_table, value);
        }
        self.areas.push(map_area); //插入段管理器中
    }
    pub fn insert_framed_area(
        &mut self,
        start_addr: VirtAddr,
        end_addr: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_addr, end_addr, MapType::Framed, permission),
            None,
        );
    }

    pub fn remove_from_startaddr(&mut self, startaddr: VirtAddr) {
        //从一个起始地址找到对应的段，将这个段对应的页删除
        let virtpage: VirtPageNum = startaddr.into(); //转换为虚拟页号
        if let Some((index, area)) = self
            .areas
            .iter_mut()
            .enumerate() //根据每一个内存区域的起始页号找到对应的area
            .find(|(_index, maparea)| maparea.vpn_range.get_start() == virtpage)
        {
            area.unmap(&mut self.page_table); //解除原来的映射
            self.areas.remove(index); //从area中将其删除
        }
    }
    fn new_kernel() -> Self {
        //生成内核的地址空间
        let mut memoryset = MemorySet::new_bare();
        //映射跳板
        memoryset.map_trampoline();

        //恒等映射各个逻辑段
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        memoryset.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memoryset.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memoryset.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memoryset.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physcial memory");
        //内核程序使用后剩余的内存
        //我们在编写代码的过程中需要直接获取内存的内容，比如在加载应用的时候
        memoryset.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping mmio");
        for &pair in MMIO {
            memoryset.push(
                MapArea::new(
                    (pair.0).into(),
                    (pair.0 + pair.1).into(),
                    MapType::Identical,
                    MapPermission::R|MapPermission::W,
                ),
                None
            );
        };

        memoryset
    }
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        //解析elf文件，生成应用程序的地址空间
        // INFO!("[kernel] from_elf...");
        let mut memoryset = MemorySet::new_bare();
        // INFO!("[kernel] mapping trampoline...");
        memoryset.map_trampoline(); //映射跳板
        let elf = ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header; //elf头
        let elf_magic = elf_header.pt1.magic; //魔数，用来判断是否是elf文件
        assert_eq!(elf_magic, [0x7f, 0x45, 0x4c, 0x46], "This is not elf file");
        // INFO!("[kernel] elf_magic is ok");
        //program header内的信息有大小，偏移量
        //以程序执行的角度看待文件
        let ph_count = elf_header.pt2.ph_count(); //program header数量
        let mut max_end_vpn = VirtPageNum(0);

        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                //需要加载的段我们就加载到内存指定位置
                let start_addr: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_addr: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                //用户态程序
                let mut map_perm = MapPermission::U;
                //执行权限
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                //申请段空间来存储

                let map_area = MapArea::new(start_addr, end_addr, MapType::Framed, map_perm);

                max_end_vpn = map_area.vpn_range.get_end();
                memoryset.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        //映射用户栈
        let max_end_va: VirtAddr = max_end_vpn.into(); //最后一页
        let mut user_stack_buttom: usize = max_end_va.into(); //
                                                              //放置一个guard page ？
        user_stack_buttom += PAGE_SIZE;
        let user_stack_base = user_stack_buttom + USER_STACK_SIZE;

        //返回应用程序的地址空间与用户栈顶以及程序入口地址
        (
            memoryset,
            user_stack_base,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn from_existed_memset(src_memset: &MemorySet) -> Self {
        //从一个已经存在的地址空间拷贝一份
        let mut memoryset = MemorySet::new_bare();
        memoryset.map_trampoline(); //映射跳板页，跳板页并没有加入到地址空间中，需要单独映射
        for area in src_memset.areas.iter() {
            let new_area = MapArea::copy_from_other(area); //拷贝一个maparea
            memoryset.push(new_area, None); //
            for vpn in area.vpn_range {
                let src_data = src_memset.translate(vpn).unwrap().ppn(); //获取父进程的虚拟页的对应的物理页
                let dis_data = memoryset.translate(vpn).unwrap().ppn(); //获取子进程虚拟页对应的物理页
                dis_data
                    .get_bytes_array() //获取字节数组
                    .copy_from_slice(src_data.get_bytes_array()); //拷贝数据
            }
        }
        memoryset
    }
    fn map_trampoline(&mut self) {
        //映射跳板
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }
    pub fn clear_area_data(&mut self) {
        self.areas.clear() //回收所有的段
    }
}

impl MapArea {
    pub fn new(
        start_addr: VirtAddr,
        end_addr: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        //为一个应用程序段分配虚拟页号

        //起始地址对应的虚拟页号
        let start_page_num = start_addr.floor();
        //结束地址对应的虚拟页号
        let end_page_num = end_addr.ceil();

        Self {
            vpn_range: VPNRange::new(start_page_num, end_page_num),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    pub fn copy_from_other(old_maparea: &MapArea) -> Self {
        //拷贝另一个地址段的相关信息
        //包括读写权限，映射方式
        Self {
            vpn_range: VPNRange::new(
                old_maparea.vpn_range.get_start(),
                old_maparea.vpn_range.get_end(),
            ),
            data_frames: BTreeMap::new(),
            map_perm: old_maparea.map_perm,
            map_type: old_maparea.map_type,
        }
    }
    fn map(&mut self, page_table: &mut PageTable) {
        //段需要管理自己的虚拟页号和物理页号
        //将这些数据写入所属应用程序的页表中
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    fn unmap(&mut self, page_table: &mut PageTable) {
        //删除这个段对应的映射关系
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        //向这个段映射的物理页面上写入数据
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0; //一次写入一个页面的数据
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len(); //数据长度
        loop {
            let src_data = &data[start..len.min(start + PAGE_SIZE)];
            let dst_data = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src_data.len()];
            dst_data.copy_from_slice(src_data); //拷贝数据
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
    fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        // 添加映射关系
        let ppn: PhysPageNum;
        match self.map_type {
            //根据映射方式选择不同的方法
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn; //物理页帧号
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        //构造一个页表项并插入页表中
        page_table.map(vpn, ppn, pte_flags);
    }
    fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        //解除映射关系
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unmap(vpn);
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

#[allow(unused)]
pub fn remap_test() {
    //测试内核映射的正确性
    //这里会使用在page_table中定义find_pte函数
    let mut kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();

    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_text.floor())
            .unwrap()
            .writable(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_rodata.floor())
            .unwrap()
            .writable(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_data.floor())
            .unwrap()
            .executable(),
        false
    );
    INFO!("The remap_test passed!");
}
