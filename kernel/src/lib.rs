#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(unique)]
#![feature(slice_rotate)]
#![feature(try_from)]
#![feature(type_ascription)]

extern crate rlibc;
extern crate volatile;
extern crate spin;

mod lang;
mod io;
#[macro_use]
mod vga;
mod ps2_io;
mod ps2;

use vga::Color;

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    vga::WRITER.lock().fill_screen(Color::Black);
    println!("Flower kernel boot");
    ps2::PS2.lock().initialize();

    halt()
}

fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
