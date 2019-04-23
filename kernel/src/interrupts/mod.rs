//! Module for interrupt handling/IDT

use x86_64::structures::idt::{InterruptDescriptorTable, ExceptionStackFrame, PageFaultErrorCode};
use interrupts::exceptions::page_fault;
use gdt;

mod legacy_pic;
mod exceptions;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        unsafe {
            idt.divide_by_zero.set_handler_fn(exceptions::divide_by_zero)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.breakpoint.set_handler_fn(exceptions::breakpoint)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.overflow.set_handler_fn(exceptions::overflow)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.bound_range_exceeded.set_handler_fn(exceptions::out_of_bounds)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.invalid_opcode.set_handler_fn(exceptions::invalid_opcode)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.device_not_available.set_handler_fn(exceptions::device_not_available)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.double_fault.set_handler_fn(exceptions::double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
            idt.invalid_tss.set_handler_fn(exceptions::invalid_tss)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.segment_not_present.set_handler_fn(exceptions::segment_not_present)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.stack_segment_fault.set_handler_fn(exceptions::stack_segment_fault)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.general_protection_fault.set_handler_fn(exceptions::general_protection_fault)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);

            let page_fault: extern "x86-interrupt" fn(&mut ExceptionStackFrame, u64) = page_fault;
            let page_fault:  extern "x86-interrupt" fn(&mut ExceptionStackFrame, PageFaultErrorCode)
                = unsafe{core::mem::transmute(page_fault)};

            unsafe {
                idt.page_fault.set_handler_fn(page_fault);
            }

            idt.x87_floating_point.set_handler_fn(exceptions::x87_floating_point)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.alignment_check.set_handler_fn(exceptions::alignment_check)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.machine_check.set_handler_fn(exceptions::machine_check)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.simd_floating_point.set_handler_fn(exceptions::simd_floating_point)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.virtualization.set_handler_fn(exceptions::virtualization)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
            idt.security_exception.set_handler_fn(exceptions::security_exception)
                .set_stack_index(gdt::PANICKING_EXCEPTION_IST_INDEX);
        }
        idt
    };
}

/// Implicitly invoke the lazy initializer of the IDT & load it, as well as disable PICs and set up
/// APICs
pub fn init() {
    info!("interrupts: initialising");
    debug!("interrupts: loading idt");
    IDT.load();
    debug!("interrupts: disabling legacy PICs");
    legacy_pic::CHAINED_PICS.lock().remap_and_disable();
    info!("interrupts: initialised")
}
