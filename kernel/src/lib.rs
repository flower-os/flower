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
extern crate x86_64;

mod lang;
mod io;
mod keyboard;
mod keymap;
#[macro_use]
mod drivers;

use drivers::vga::{self, VgaColor, Color};
use keyboard::{Keyboard, Ps2Keyboard};

const FLOWER: &'static str = include_str!("resources/art/flower.txt");
const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");


/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    vga::WRITER.lock().fill_screen(Color::Black);

    // Print flower
    vga::WRITER.lock().set_color(
        VgaColor::new(Color::LightBlue, Color::Black)
    );
    print!("\n{}", FLOWER);
    vga::WRITER.lock().set_color(
        VgaColor::new(Color::Green, Color::Black)
    );
    print!("{}", FLOWER_STEM);

    // Reset colors
    vga::WRITER.lock().set_color(
        VgaColor::new(Color::White, Color::Black)
    );


    // Reset cursor position to (0, 0)
    // It's hackish but it looks better
    vga::WRITER.lock().set_cursor_pos((0, 0));

    // Print boot message
    vga::WRITER.lock().write_str_colored(
        "Flower kernel boot!\n-------------------\n\n",
         VgaColor::new(Color::Green, Color::Black)
    ).expect("Color code should be valid");

    drivers::ps2::PS2.lock().initialize();

    let keyboard_device = &mut ps2::PS2.lock().devices[0];
    let mut keyboard = Ps2Keyboard::new(keyboard_device);
    println!("Enabling keyboard");
    if keyboard.enable() {
        println!("Successfully enabled keyboard");
        loop {
            if let Some(char) = keyboard.read_char() {
                print!("{}", char);
            }
        }
    } else {
        println!("Failed to enable keyboard");
    }

    halt()
}

// TODO move somewhere else
fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
