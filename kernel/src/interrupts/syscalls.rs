use x86_64::structures::idt::ExceptionStackFrame;

use syscalls::syscall_handler;

pub extern "x86-interrupt" fn syscall_int(stack_frame: &mut ExceptionStackFrame) {
    syscall_handler();
}