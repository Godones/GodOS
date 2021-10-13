.altmacro
.macro SAVE_SN n
    sd s\n, (\n+1)*8(sp)
.endm
.macro LOAD_SN n
    ld s\n, (\n+1)*8(sp)
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr2: &*const TaskContext, 当前trap控制流
    #     next_task_cx_ptr2: &*const TaskContext 切换目的trap控制流
    # )
    # push TaskContext to current sp and save its address to where a0 points to
    # current_task_cx_ptr2位于a0 寄存器
    # next_task_cx_ptr2 位于a1寄存器

    addi sp, sp, -13*8 #开辟一部分空间保存当前的现场
    sd sp, 0(a0) #将sp的值放入a0中,
    # fill TaskContext with ra & s0-s11
    sd ra, 0(sp) #将ra返回地址保存在sp指向的栈顶上，ra的地址其实就是要接着指向的位置
    .set n, 0 #保存s0-s11
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # ready for loading TaskContext a1 points to
    ld sp, 0(a1) #将a1的值加载到sp，即此时sp指向下一个task栈顶
    # load registers in the TaskContext
    ld ra, 0(sp) #将下一个task栈顶保存在ra位置里面，即原来保存的指令位置
    .set n, 0 #恢复寄存器
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # pop TaskContext
    addi sp, sp, 13*8 #恢复sp位置
    ret #函数返回，此时pc就会读取ra的内容了