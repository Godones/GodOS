use crate::file::{open_file, File, Mail, OpenFlags, Stdin, Stdout};
use crate::mm::page_table::translated_refmut;
use crate::mm::{MemorySet, KERNEL_SPACE};
use crate::my_struct::my_ref_cell::MyRefCell;
use crate::sync::{Monitor, Mutex, Semaphore};
use crate::task::add_task;
use crate::task::id::{pid_alloc, PidHandle, RecycleAllocator};
use crate::task::task::TaskControlBlock;
use crate::trap::context::TrapFrame;
use crate::trap::trap_handler;
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;
///! 进程控制块定义
pub struct ProcessControlBlock {
    //不可变数据
    pub pid: PidHandle,
    //可变数据
    inner: MyRefCell<ProcessControlBlockInner>,
}
pub struct ProcessControlBlockInner {
    pub is_zombie: bool,
    pub memory_set: MemorySet,                     //任务地址空间
    pub parent: Option<Weak<ProcessControlBlock>>, //父进程
    pub exit_code: i32,
    pub children: Vec<Arc<ProcessControlBlock>>, //子进程需要引用计数
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>, //文件描述符表
    /// 文件描述符 (File Descriptor) 代表了一个特定读写属性的I/O资源。
    pub task: Vec<Option<Arc<TaskControlBlock>>>, //线程管理器
    pub task_res_allocator: RecycleAllocator,    //升级版分配器
    pub mutex_list: Vec<Option<Arc<dyn Mutex>>>, //记录进程拥有的互斥资源
    pub semaphore_list: Vec<Option<Arc<Semaphore>>>, //记录信号量资源
    pub monitor_list: Vec<Option<Arc<Monitor>>>, //记录管程资源
}

impl ProcessControlBlockInner {
    #[allow(unused)]
    pub fn get_user_token(&self) -> usize {
        //获取当前任务的用户地址空间根页表satp
        self.memory_set.token()
    }
    pub fn is_zombie(&self) -> bool {
        //查看是否是僵尸进程
        self.is_zombie
    }
    pub fn get_one_fd(&mut self) -> usize {
        //查看文件描述符表获取一个最小的描述符
        if let Some(fd) = (0..self.fd_table.len()).find(|x| self.fd_table[*x].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
    pub fn alloc_tid(&mut self) -> usize {
        // 申请一个tid：线程描述符
        self.task_res_allocator.alloc()
    }
    pub fn dealloc_tid(&mut self, tid: usize) {
        //回收线程的tid
        self.task_res_allocator.dealloc(tid)
    }
    pub fn thread_count(&self) -> usize {
        //获取线程数目
        self.task.len()
    }
    pub fn get_task(&self, tid: usize) -> Arc<TaskControlBlock> {
        //根据tid获取线程
        self.task[tid].as_ref().unwrap().clone()
    }
}

impl ProcessControlBlock {
    pub fn new(data: &[u8]) -> Arc<Self> {
        //构造用户地址空间
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(data);
        //为进程分配pid
        let pid = pid_alloc();
        let process = Arc::new(Self {
            pid,
            inner: MyRefCell::new(ProcessControlBlockInner {
                is_zombie: false,
                memory_set,
                exit_code: 0,
                parent: None,
                children: Vec::new(),
                fd_table: vec![
                    Some(Arc::new(Stdin)),
                    Some(Arc::new(Stdout)),
                    Some(Arc::new(Stdout)),
                    Some(Mail::new()), //邮箱文件描述符
                ],
                task: Vec::new(),
                task_res_allocator: RecycleAllocator::new(),
                mutex_list: Vec::new(),
                semaphore_list: Vec::new(),
                monitor_list: Vec::new(),
            }),
        }); //构造任务控制块
            //创建主线程
        let main_task = Arc::new(TaskControlBlock::new(process.clone(), ustack_base, true));
        //创建主线程的trap上下文
        let task_inner = main_task.get_inner_access();
        let trap_cx = task_inner.get_trap_cx(); //获取trap上下文
                                                //获取用户栈顶
        let user_stack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kernel_stack_top = main_task.kernel_stack.get_stack_top(); //内核栈顶
        drop(task_inner);
        *trap_cx = TrapFrame::app_into_context(
            entry_point,
            user_stack_top,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        let mut process_inner = process.get_inner_access();
        //加入线程管理列表中
        process_inner.task.push(Some(main_task.clone()));
        drop(process_inner);
        add_task(main_task); //加入等待队列上面
        process
    }
    pub fn get_inner_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        //获取内部数据的可变借用
        self.inner.get_mut()
    }
    pub fn get_pid(&self) -> usize {
        self.pid.0
    }
    pub fn spawn(self: &Arc<ProcessControlBlock>, path: &str) -> isize {
        //直接创建一个新的子进程，并且执行程序
        let node = open_file(path, OpenFlags::R).unwrap();
        let data = node.read_all();
        if data.len() != 0 {
            //这里直接new一个新的进程，会创建主线程
            let process_control_block = ProcessControlBlock::new(data.as_slice());
            //修改其父进程的引用
            let mut inner = process_control_block.get_inner_access();
            inner.parent = Some(Arc::downgrade(self));
            self.get_inner_access()
                .children
                .push(process_control_block.clone());
            drop(inner);
            let pid = process_control_block.get_pid() as isize;
            pid
        } else {
            return -1;
        }
    }

    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        //只支持单线程的进程
        //更换当前进程的数据
        assert_eq!(self.get_inner_access().thread_count(), 1); //判断当前进程是否只有一个子线程
        let (memoryset, user_stack_base, entry_point) = MemorySet::from_elf(elf_data);
        //在应用栈中开辟空间用来存放传入的参数
        //开辟几个存放地址的空间，这几个地址会指向更低地址存放的参数
        let token = memoryset.token();
        //更换地址空间
        self.get_inner_access().memory_set = memoryset;
        //为主线程申请资源
        let main_task = self.get_inner_access().get_task(0); //主线程
        let mut main_task_inner = main_task.get_inner_access();
        //切换主线程的相关内容
        main_task_inner.res.as_mut().unwrap().ustack_base = user_stack_base;
        main_task_inner.res.as_mut().unwrap().alloc_user_res(); //重新申请资源
        main_task_inner.trap_cx_ppn = main_task_inner.res.as_mut().unwrap().trap_cx_ppn();
        let mut user_sp = main_task_inner.res.as_ref().unwrap().ustack_top()
            - (args.len() + 1) * core::mem::size_of::<usize>();
        let arg_base = user_sp;
        let mut arg_vec: Vec<_> = (0..=args.len())
            .map(|index| {
                translated_refmut(
                    token,
                    (arg_base + index * core::mem::size_of::<usize>()) as *mut usize,
                )
            })
            .collect();
        *arg_vec[args.len()] = 0; //最高地址处设为0
        for i in 0..args.len() {
            //存放相关参数
            //将sp指针下移存放实际的参数，arg_base上移存放指针指向sp
            user_sp -= args[i].len() + 1;
            *arg_vec[i] = user_sp;
            let mut p = user_sp;
            for ch in args[i].as_bytes() {
                //存入一个个字符
                *translated_refmut(token, p as *mut u8) = *ch;
                p += 1;
            }
            *translated_refmut(token, p as *mut u8) = 0; //字符串结束标记
        }
        user_sp -= user_sp % core::mem::size_of::<usize>(); //对齐8字节

        let mut trap_cx = TrapFrame::app_into_context(
            entry_point, //新的入口
            user_sp,     //新的用户栈
            KERNEL_SPACE.lock().token(),
            main_task.kernel_stack.get_stack_top(), //原有的内核栈
            trap_handler as usize,
        );
        trap_cx.reg[10] = args.len(); //参数长度
        trap_cx.reg[11] = arg_base; //参数起始位置
        *main_task_inner.get_trap_cx() = trap_cx;
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        //fork一个新的进程
        //只支持单个线程的进程
        //构造用户地址空间
        let mut parent_inner = self.get_inner_access();
        assert_eq!(parent_inner.thread_count(), 1);

        //复制地址空间已经数据
        let memory_set = MemorySet::from_existed_memset(&parent_inner.memory_set);
        //为进程分配pid
        let pid = pid_alloc();
        //copy父进程的文件描述符表
        let mut new_fdtable: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(f) = fd {
                new_fdtable.push(Some(Arc::clone(f)));
            } else {
                new_fdtable.push(None); //文件描述符为空，代表此类型资源不可用
            }
        }
        let child = Arc::new(Self {
            pid,
            inner: MyRefCell::new(ProcessControlBlockInner {
                is_zombie: false,
                memory_set,
                exit_code: 0,
                parent: Some(Arc::downgrade(self)), //弱引用
                children: Vec::new(),
                fd_table: new_fdtable,
                task: Vec::new(),
                task_res_allocator: RecycleAllocator::new(),
                mutex_list: Vec::new(),
                semaphore_list: Vec::new(),
                monitor_list: Vec::new(),
            }),
        }); //构造任务控制块
            //加入子进程中
        parent_inner.children.push(child.clone());
        //创建子进程的主线程
        let main_task = Arc::new(TaskControlBlock::new(
            child.clone(),
            parent_inner
                .get_task(0)
                .get_inner_access()
                .res
                .as_ref()
                .unwrap()
                .ustack_base,
            false,
        )); //不需要重新申请用户栈和trap上下文

        let mut child_inner = child.get_inner_access();
        child_inner.task.push(Some(Arc::clone(&main_task))); //将线程加入线程队列中
        drop(child_inner);
        let main_task_inner = main_task.get_inner_access();

        main_task_inner.get_trap_cx().kernel_sp = main_task.kernel_stack.get_stack_top();
        drop(main_task_inner);
        add_task(main_task); //加入等待队列上面
        child
    }
}
