//! Module for interrupt handling/IDT

use x86_64::structures::idt::Idt;

mod pic;
mod exceptions;
mod handlers;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        init_exception_handlers(&mut idt);
        init_irq_handlers(&mut idt);
        idt
    };
}

#[repr(u8)]
enum StandardIrq {
    Pit = 0,
    Ps2Keyboard = 1,
}

impl Into<u8> for StandardIrq {
    fn into(self) -> u8 { self as u8 }
}

impl StandardIrq {
    fn handle(self, handler: fn()) {
        // TODO: This is a magic call. It makes things compile. Do not question magic.
        ::drivers::ps2::CONTROLLER.lock();

        pic::CHAINED_PICS.lock().handle_interrupt(self as u8, handler).unwrap();
    }
}

/// Setup IDTs and initialize and remap PICs
pub fn initialize() {
    info!("interrupts: initializing");

    IDT.load();
    debug!("interrupts: initialized idt");

    pic::CHAINED_PICS.lock().init_and_remap();
    debug!("interrupts: pic initialized and remapped");
}

pub fn enable() {
    unsafe { asm!("sti" :::: "volatile"); }
}

pub fn disable() {
    unsafe { asm!("cli" :::: "volatile"); }
}

fn init_exception_handlers(idt: &mut Idt) {
    idt.divide_by_zero.set_handler_fn(exceptions::divide_by_zero);
    idt.breakpoint.set_handler_fn(exceptions::breakpoint);
    idt.overflow.set_handler_fn(exceptions::overflow);
    idt.bound_range_exceeded.set_handler_fn(exceptions::out_of_bounds);
    idt.invalid_opcode.set_handler_fn(exceptions::invalid_opcode);
    idt.device_not_available.set_handler_fn(exceptions::device_not_available);
    idt.double_fault.set_handler_fn(exceptions::double_fault);
    idt.invalid_tss.set_handler_fn(exceptions::invalid_tss);
    idt.segment_not_present.set_handler_fn(exceptions::segment_not_present);
    idt.stack_segment_fault.set_handler_fn(exceptions::stack_segment_fault);
    idt.general_protection_fault.set_handler_fn(exceptions::general_protection_fault);
    idt.page_fault.set_handler_fn(exceptions::page_fault);
    idt.x87_floating_point.set_handler_fn(exceptions::x87_floating_point);
    idt.alignment_check.set_handler_fn(exceptions::alignment_check);
    idt.machine_check.set_handler_fn(exceptions::machine_check);
    idt.simd_floating_point.set_handler_fn(exceptions::simd_floating_point);
    idt.virtualization.set_handler_fn(exceptions::virtualization);
    idt.security_exception.set_handler_fn(exceptions::security_exception);
}

fn init_irq_handlers(idt: &mut Idt) {
    idt.interrupts[StandardIrq::Pit as usize].set_handler_fn(handlers::irq_pit);
    idt.interrupts[StandardIrq::Ps2Keyboard as usize].set_handler_fn(handlers::irq_kbd);
}
