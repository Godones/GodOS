OUTPUT_ARCH(riscv)/*指定链接脚本*/
ENTRY(_start) /*程序入口*/
BASE_ADDRESS = 0X80200000 /* 基本变量，基准地址*/
/*描述输出文件的内存布局*/
SECTIONS
{
    . = BASE_ADDRESS;
    /*其中 . 表示当前地址，也就是链接器会从它指向的位置开始往下放置从输入的目标文件 中收集来的段*/
    skernel = .;

    stext = .;
    /*
        冒号前面表示最终生成的可执行文件的一个段的名字，花括号内按照放置顺
        序描述将所有输入目标文件的哪些段放在这个段中
     */
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
    }

    . = ALIGN(4K);
    edata = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}