//! Module for interrupt handling/IDT

use x86_64::structures::idt::{Idt, ExceptionStackFrame};

use alloc::vec::Vec;
use spin::RwLock;
use array_init;

mod pic;
mod exceptions;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        init_interrupt_handlers(&mut idt);
        idt
    };
}

lazy_static! {
    static ref LISTENERS: RwLock<[Vec<fn()>; 16]> = RwLock::new(
        array_init::array_init(|_| Vec::with_capacity(1))
    );
}

/// Registers a listener for the given IRQ
pub fn listen<I: Into<u8>>(irq: I, listener: fn()) {
    LISTENERS.write()[irq.into() as usize].push(listener);
}

/// Dispatches the given IRQ to all relevant registered listeners
pub fn dispatch_irq(irq: u8) {
    for listener in LISTENERS.read()[irq as usize].iter() {
        listener();
    }
}

#[repr(u8)]
pub enum Irq {
    Pit = 0,
    Ps2Keyboard = 1,
    Ps2Mouse = 12,
}

impl Into<u8> for Irq {
    fn into(self) -> u8 { self as u8 }
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

macro_rules! init_irq_handlers {
    ($idt:expr, $($irq:expr),*) => {
        $(
            {
                extern "x86-interrupt" fn handle_irq(_: &mut ExceptionStackFrame) {
                    pic::CHAINED_PICS.lock().handle_interrupt($irq, || dispatch_irq($irq));
                }
                $idt.interrupts[$irq].set_handler_fn(handle_irq);
            }
        )*
    };
}

fn init_interrupt_handlers(idt: &mut Idt) {
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

    init_irq_handlers!(idt, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
}
