mod test;

#[no_mangle]
pub extern fn syscall_callback() {
    syscall_handler();
    unsafe{asm!("sysret");}
}

pub fn syscall_handler() {
    let mut id: usize;
    unsafe{asm!("nop" : "={rax}"(id))}

    match id {
         0 => test::ping(),
         _ => {},
    };

    let mut rcx: u64;
}