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
#[macro_use]
mod vga;

use vga::Color;

const FLOWER: &'static str = include_str!("flower.txt");
const FLOWER_STEM: &'static str = include_str!("flower_stem.txt");

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    vga::WRITER.lock().fill_screen(Color::Black);
    println!("Flower kernel boot");

    // Print flower
    vga::WRITER.lock().set_color(
        vga::VgaColor::new(Color::LightBlue, Color::Black)
    );
    print!("{}", FLOWER);
    vga::WRITER.lock().set_color(
        vga::VgaColor::new(Color::Green, Color::Black)
    );
    print!("{}", FLOWER_STEM);

    // Reset colors
    vga::WRITER.lock().set_color(
        vga::VgaColor::new(Color::White, Color::Black)
    );

    unsafe { asm!("hlt"); }
    loop {}
}
