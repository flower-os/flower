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

const FLOWER: &'static str = include_str!("flower.txt");
const FLOWER_STEM: &'static str = include_str!("flower_stem.txt");

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    vga::WRITER.lock().fill_screen(Color::Black);

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

    // Reset cursor position to (0, 0)
    // It's hackish but it looks better
    vga::WRITER.lock().set_cursor_pos((0, 0));


    println!("Flower kernel boot");
    ps2::PS2.lock().initialize();



    halt()
}

fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
