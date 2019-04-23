//! Exception handlers

use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};

pub extern "x86-interrupt" fn divide_by_zero(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: divide by zero\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: breakpoint\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: overflow\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn out_of_bounds(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: out of bounds\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: &mut ExceptionStackFrame) {
    panic!(
        "cpuex: invalid opcode \n{:#?}\n => note: qword at {:?} is 0x{:x}",
        stack_frame,
        stack_frame.instruction_pointer,
        unsafe { *(stack_frame.instruction_pointer.as_ptr::<u64>()) },
    );
}

pub extern "x86-interrupt" fn device_not_available(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: device not available\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: double fault {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: invalid tss {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn segment_not_present(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: segment not present {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn stack_segment_fault(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: stack segment fault {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn general_protection_fault(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: general protection fault {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn page_fault(stack_frame: &mut ExceptionStackFrame, code: PageFaultErrorCode) {
    let cr2: u64;
    unsafe { asm!("mov %cr2, $0" : "=r" (cr2)); }
    panic!(
        "cpuex: page fault {:?}\n{:#?}\n => note: CR2 = 0x{:x}\
        \n Check that this address is mapped correctly",
        code, stack_frame, cr2
    );
}

pub extern "x86-interrupt" fn x87_floating_point(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: x87 floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: alignment check {}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn machine_check(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: machine check\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_floating_point(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: simd floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization(stack_frame: &mut ExceptionStackFrame) {
    panic!("cpuex: virtualization\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn security_exception(stack_frame: &mut ExceptionStackFrame, code: u64) {
    panic!("cpuex: security exception {}\n{:#?}", code, stack_frame);
}
