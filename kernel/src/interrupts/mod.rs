//! Module for interrupt handling/IDT

use x86_64::structures::idt::Idt;

mod legacy_pic;
mod exceptions;
mod irqs;

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(exceptions::breakpoint);
        idt
    };
}

/// Implicitly invoke the lazy initializer of the IDT & load it, as well as disable PICs and set up
/// APICs
pub fn init() {
    IDT.load();
    legacy_pic::CHAINED_PICS.lock().remap_and_disable();
}