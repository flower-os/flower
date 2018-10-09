mod test;

pub fn syscall_handler() {
    let mut id: usize;
    unsafe{asm!("nop" : "={rax}"(id))}

    match id {
         0 => test::ping(),
         _ => {},
    }
}