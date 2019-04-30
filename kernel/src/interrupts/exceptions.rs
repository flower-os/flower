//! Exception handlers

use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

pub extern "x86-interrupt" fn divide_by_zero(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: divide by zero\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn debug(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: debug\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn nmi(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: nmi\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: breakpoint\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: overflow\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn out_of_bounds(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: out of bounds\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: &mut InterruptStackFrame) {
    panic!(
        "cpuex: invalid opcode \n{:#?}\n => note: qword at {:?} is 0x{:x}",
        stack_frame,
        stack_frame.instruction_pointer,
        unsafe { *(stack_frame.instruction_pointer.as_ptr::<u64>()) },
    );
}

pub extern "x86-interrupt" fn device_not_available(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: device not available\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: &mut InterruptStackFrame, code: u64) {
    panic!("cpuex: double fault 0x{:x}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss(stack_frame: &mut InterruptStackFrame, code: u64) {
    panic!("cpuex: invalid tss 0x{:x}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn segment_not_present(stack_frame: &mut InterruptStackFrame, code: u64) {
    panic!("cpuex: segment not present 0x{:x}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn stack_segment_fault(stack_frame: &mut InterruptStackFrame, code: u64) {
    panic!("cpuex: stack segment fault 0x{:x}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn general_protection_fault(stack_frame: &mut InterruptStackFrame, code: u64) {
    // WORKAROUND: https://github.com/rust-lang/rust/issues/57270
    // Workaround code taken from https://github.com/phil-opp/blog_os/issues/513
    unsafe{
        asm!("sub rsp, 8
              sub rbp, 8"::::"intel", "volatile");
    }

    let error_code = code;

    unsafe{
        asm!("add rsp, 8
              add rbp, 8"::::"intel", "volatile");
    }

    panic!("cpuex: general protection fault 0x{:x}\n{:#?}", error_code, stack_frame);
}

pub extern "x86-interrupt" fn page_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    // WORKAROUND: https://github.com/rust-lang/rust/issues/57270
    // Workaround code taken from https://github.com/phil-opp/blog_os/issues/513
    unsafe{
        asm!("sub rsp, 8
              sub rbp, 8"::::"intel", "volatile");
    }

    let error_code = PageFaultErrorCode::from_bits_truncate(error_code).clone();

    unsafe{
        asm!("add rsp, 8
              add rbp, 8"::::"intel", "volatile");
    }

    let cr2: u64;
    unsafe { asm!("mov %cr2, $0" : "=r" (cr2)); }

    panic!(
        "cpuex: page fault (flags: {:?})\n{:#?}\n => note: CR2 = 0x{:x}\
    \n Check that this address is mapped correctly",
        error_code, stack_frame, cr2
    );
}

pub extern "x86-interrupt" fn x87_floating_point(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: x87 floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check(stack_frame: &mut InterruptStackFrame, code: u64) {
    panic!("cpuex: alignment check 0x{:x}\n{:#?}", code, stack_frame);
}

pub extern "x86-interrupt" fn machine_check(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: machine check\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_floating_point(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: simd floating point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization(stack_frame: &mut InterruptStackFrame) {
    panic!("cpuex: virtualization\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn security_exception(stack_frame: &mut InterruptStackFrame, code: u64) {
    panic!("cpuex: security exception 0x{:x}\n{:#?}", code, stack_frame);
}
