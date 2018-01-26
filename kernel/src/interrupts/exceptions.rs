//! Exception handlers

use x86_64::structures::idt::ExceptionStackFrame;

pub extern "x86-interrupt" fn breakpoint(
    stack_frame: &mut ExceptionStackFrame)
{
    // TODO eprintln macro
    println!("Cpu Exception: Breakpoint\n{:#?}", stack_frame);
}
