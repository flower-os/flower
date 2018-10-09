use x86_64::structures::idt::ExceptionStackFrame;

pub extern "x86-interrupt" fn syscall(stack_frame: &mut ExceptionStackFrame) {
    let mut id: u64;
    unsafe {
        asm!("nop":"={rax}"(id):::);
    }
    info!("Syscall: {}", id);
}