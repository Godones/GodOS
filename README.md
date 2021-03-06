### GodOS

项目简述：清华大学操作系统实验`rCore`实验记录。

项目初衷：学习操作系统知识，想写一个操作系统。

目标：完成`rCore`中9个实验代码的编写和章节测试。

#### 进度

+ [x] 第一章: 移除标准库，实现屏幕输出功能
+ [x] 第二章: 支持多个任务依次执行的批处理系统
+ [x] 第三章: 支持多道程序与分时多任务
+ [x] 第四章: 完成操作系统第一个抽象: 虚拟内存--地址空间--->主存
+ [x] 第五章: 完成操作系统第二个抽象: 进程-->CPU
+ [x] 第六章: 完成操作系统第三个抽象: 文件系统--> I/O设备
+ [x] 第七章: 支持基于管道的进程间通行
+ [x] 第八章: 支持线程
+ [x] 第九章: 介绍I/O设备的抽象过程

#### 实验报告(只包含1-5章)

 [实验1报告.pdf](doc/实验1报告.pdf) 

 [实验2报告.pdf](doc/实验2报告.pdf) 

 [实验3报告.pdf](doc/实验3报告.pdf) 

 [实验4报告.pdf](doc/实验4报告.pdf) 

 [实验5报告.pdf](doc/实验5报告.pdf) 

实验报告5包含当时系统的大致结构图。

#### 一些杂七杂八的文件

 [RISC-V-Reader-Chinese-v2p1.pdf](doc/RISC-V-Reader-Chinese-v2p1.pdf) 

 [一点汇编知识.pdf](doc/一点汇编知识.pdf) 

 [riscv-sbi.pdf](doc/riscv-sbi.pdf) 

#### 对实验的一些建议

在做实验的过程中，由于实验已经更新多次，因此在前面章节遇到的问题可能很多已经被解决。因此主要针对后面的章节。

1. 希望第九章驱动设备对如何加入当前的系统再添加一点详细的说明或者知识链接，或者挑选一个外部设备，对其整个开发过程做一个介绍。
2. 第八章并发部分对线程实现的细节不太多，需要阅读源码，系统的系统调用部分和之前的进程部分都有多处修改，需要注意一些细节代码，希望实验加入线程实现的部分说明或提醒
3. 文件系统部分设计结构很好，要是后面能支持多级目录就更好了，或者当做作业也行。
4. 希望对RustSBI做一点介绍，因为当前系统的启动建立在其之上。

#### 未来工作

- [x] `Buddy System Allocator`实现
- [ ] 文件系统改进(多级目录+显示)
- [ ] 进程和线程改进
