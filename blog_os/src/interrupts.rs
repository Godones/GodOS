use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;

use lazy_static::lazy_static;

lazy_static! {
    static ref IDT:InterruptDescriptorTable = {
        //中断描述符表
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
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

