//! Exception handlers

use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};

pub extern "x86-interrupt" fn divide_by_zero(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: divide by zero\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: breakpoint\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: overflow\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn out_of_bounds(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: out of bounds\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: invalid opcode\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn device_not_available(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: device not available\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: double fault {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: invalid tss {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn segment_not_present(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: segment not present {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn stack_segment_fault(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: stack segment fault {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn general_protection_fault(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: general protection fault {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn page_fault(stack_frame: &mut ExceptionStackFrame, code: PageFaultErrorCode) {
    println!("cpuex: page fault fault {:?}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn x87_floating_point(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: x87 floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: alignment check {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn machine_check(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: machine check\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_floating_point(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: simd floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization(stack_frame: &mut ExceptionStackFrame) {
    println!("cpuex: simd floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn security_exception(stack_frame: &mut ExceptionStackFrame, code: u64) {
    println!("cpuex: security exception {}\n{:#?}", code, stack_frame);
}
