#![no_std]
#![feature(lang_items)]
#![feature(asm)]

mod lang;

fn kmain() {
    asm!("hlt");
}
