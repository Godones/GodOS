use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;

use lazy_static::lazy_static;

lazy_static! {
    static ref IDT:InterruptDescriptorTable = {
        //中断描述符表
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}
extern "x86-interrupt" fn breakpoint_handler(stack_frame:
                                             InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub fn init_idt() {
    IDT.load();//让cpu加载新的中断描述符表}
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code:u64) ->!
{
    //双重错误处理函数
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}",stack_frame);
}