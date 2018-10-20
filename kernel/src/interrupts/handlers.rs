use x86_64::structures::idt::Idt;
use x86_64::structures::idt::ExceptionStackFrame;

use spin::Mutex;

use super::{StandardIrq, pic};

pub extern "x86-interrupt" fn irq_pit(_: &mut ExceptionStackFrame) {
    StandardIrq::Pit.handle(|| {
        println!("pit");
    });
}

pub extern "x86-interrupt" fn irq_kbd(_: &mut ExceptionStackFrame) {
    StandardIrq::Ps2Keyboard.handle(|| {
        println!("kbd");
    });
}
