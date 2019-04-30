use crate::gdt::{GDT, TSS};
use x86_64::VirtAddr;

/// Jumps to usermode.
#[naked]
pub unsafe fn jump_usermode(stack_ptr: usize, instruction_ptr: usize) -> ! {
    let ds = GDT.selectors.user_ds.0;
    let cs = GDT.selectors.user_cs.0;

    asm!("
    mov ax, 0x33
    mov ds, ax
    mov es, ax
    mov fs ,ax
    mov gs, ax
    mov rsp, rbx

    mov rax, rsp
    push 0x33
    push rax
    pushfq
    " :: "{ax}"(ds), "{rbx}"(stack_ptr) :: "intel", "volatile");

    let kernel_rsp: u64;
    asm!("mov $0, rsp" : "=r"(kernel_rsp) ::: "intel");

    TSS.wait().unwrap().lock().tss.get_mut().privilege_stack_table[0] = VirtAddr::new(
        kernel_rsp
    );

    asm!("
    push 0x2b
    mov rax, rbx
    push rax
    iretq
    ":: "{rax}"(cs), "{rbx}"(instruction_ptr) :: "intel", "volatile");
    core::intrinsics::unreachable()
}
