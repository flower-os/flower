use crate::{ps2, snake, process, gdt::GDT, memory::paging::ACTIVE_PAGE_TABLES};
use x86_64::VirtAddr;

pub fn usermode_begin() -> ! {
    let pid = process::ProcessId::next();
    let process = process::Process::new();
    let page_tables = process.page_tables.clone();
    process::PROCESSES.insert(pid, process);

    ACTIVE_PAGE_TABLES.lock().switch(&page_tables);

    unsafe { jump_usermode() }
}

/// Jumps to usermode.
#[naked]
pub unsafe fn jump_usermode() -> ! {
    let ds = GDT.selectors.user_ds.0;
    let cs = GDT.selectors.user_cs.0;

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
    " :: "{ax}"(ds) :: "intel", "volatile");

    let rsp: u64;
    asm!("mov %rsp, $0" : "=r"(rsp));

    crate::gdt::TSS.wait().unwrap().lock().tss.get_mut().privilege_stack_table[0] = VirtAddr::new(rsp);

    asm!("
    push 0x2b
    mov rax, rbx
    push rax
    iretq
    ":: "{rax}"(cs), "{rbx}"(usermode as u64) :: "intel", "volatile");
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
