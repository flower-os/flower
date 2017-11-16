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
mod drivers;

use drivers::vga::{VgaColor, Color};

const FLOWER: &'static str = include_str!("resources/ascii_art/flower.txt");
const FLOWER_STEM: &'static str = include_str!("resources/ascii_art/flower_stem.txt");

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    drivers::vga::WRITER.lock().fill_screen(Color::Black);

    // Print flower
    drivers::vga::WRITER.lock().set_color(
        VgaColor::new(Color::LightBlue, Color::Black)
    );
    print!("\n{}", FLOWER);
    drivers::vga::WRITER.lock().set_color(
        VgaColor::new(Color::Green, Color::Black)
    );
    print!("{}", FLOWER_STEM);

    // Reset colors
    drivers::vga::WRITER.lock().set_color(
        VgaColor::new(Color::White, Color::Black)
    );

    // Reset cursor position to (0, 0)
    // It's hackish but it looks better
    drivers::vga::WRITER.lock().set_cursor_pos((0, 0));

    // Print boot message
    drivers::vga::WRITER.lock().write_str_colored(
        "Flower kernel boot!\n-------------------\n\n",
         VgaColor::new(Color::Green, Color::Black)
    ).expect("Color code should be valid");

    drivers::ps2::PS2.lock().initialize();

    halt()
}

// TODO move somewhere else
fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
