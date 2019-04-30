use crate::{ps2, snake};
use self::process::{Process, ProcessId, PROCESSES};

pub const STACK_TOP: usize = 0x7ffffffff000; // Top of lower half but page aligned
pub const INITIAL_STACK_SIZE_PAGES: usize = 16; // 64kib stack

pub mod process;
mod jump;

pub fn usermode_begin() -> ! {
    let pid = unsafe { Process::spawn(usermode as usize) };
    let mut process = PROCESSES.get_mut(&pid).unwrap();
    process.run()
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
