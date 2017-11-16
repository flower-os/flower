//! Module for interrupt handling/IDT

use x86_64::structures::idt::Idt;

mod exceptions;
mod irqs;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(exceptions::breakpoint);
        idt
    };
}

/// Implicitly invoke the lazy initializer of the IDT and load it
pub fn init() {
    IDT.load();
}