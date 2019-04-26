use crate::memory::paging::{PAGE_TABLES, PageSize, Page};
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

    crate::gdt::TSS.wait().unwrap().lock().get_mut().privilege_stack_table[0] = VirtAddr::new(rsp);

    asm!("
    push 0x2b
    mov rax, rbx
    push rax
    iretq
    ":: "{rbx}"(crate::usermode as u64) :: "intel", "volatile");
    core::intrinsics::unreachable()
}
