use crate::{ps2, snake};
use x86_64::VirtAddr;

/// Jumps to usermode.
#[naked]
pub unsafe fn jump_usermode() -> ! {
    asm!("
    mov ax, 0x33
    mov ds, ax
    mov es, ax
    mov fs ,ax
    mov gs, ax

    mov rax, rsp
    push 0x33
    push rax
    pushfq
    " :::: "intel", "volatile");

    let rsp: u64;
    asm!("mov %rsp, $0" : "=r"(rsp));

    crate::gdt::TSS.wait().unwrap().lock().tss.get_mut().privilege_stack_table[0] = VirtAddr::new(rsp);

    asm!("
    push 0x2b
    mov rax, rbx
    push rax
    iretq
    ":: "{rbx}"(usermode as u64) :: "intel", "volatile");
    core::intrinsics::unreachable()
}

pub extern fn usermode() -> ! {
    info!("Jumped into userspace successfully!");
    // Initialize the PS/2 controller
    let mut controller = ps2::CONTROLLER.lock();
    match controller.initialize() {
        Ok(_) => info!("ps2c: init successful"),
        Err(error) => { error!("ps2c: {:?}", error); loop {}},
    };

    snake::snake(&mut controller);
    loop {}
}