//! Module for interrupt handling/IDT

use x86_64::structures::idt::Idt;

mod pic;
mod exceptions;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
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
        idt
    };
}

/// Implicitly invoke the lazy initializer of the IDT & load it, as well as disable PICs and set up
/// APICs
pub fn initialize() {
    info!("interrupts: initializing");

    IDT.load();
    debug!("interrupts: initialized idt");

    pic::CHAINED_PICS.lock().init_and_remap();
    debug!("interrupts: pic initialized and remapped");
}
