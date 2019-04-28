use core::ptr;
use crate::{util, ps2, snake, process, gdt::GDT, memory::paging::ACTIVE_PAGE_TABLES};
use x86_64::VirtAddr;

pub const STACK_TOP: usize = 0x7ffffffff000; // Top of lower half but page aligned
pub const INITIAL_STACK_SIZE_PAGES: usize = 16; // 64kib stack

pub fn usermode_begin() -> ! {
    let pid = process::ProcessId::next();
    let process = process::Process::new();
    let page_tables = process.page_tables.clone();
    process::PROCESSES.insert(pid, process);

    ACTIVE_PAGE_TABLES.lock().switch(&page_tables);

    let stack_top = STACK_TOP;
    let stack_size = INITIAL_STACK_SIZE_PAGES * 0x1000;
    let stack_bottom = stack_top - stack_size;

    // TODO
    trace!("stack bottom = 0x{:x}", stack_bottom);

    unsafe {
        // Zero the stack
        util::memset_volatile(stack_bottom as *mut u8, 0, stack_size);
        jump_usermode()
    }
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
    mov rsp, rbx

    mov rax, rsp
    push 0x33
    push rax
    pushfq
    " :: "{ax}"(ds), "{rbx}"(STACK_TOP) :: "intel", "volatile");

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
