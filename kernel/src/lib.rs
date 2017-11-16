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
mod keyboard;
mod keymap;
mod ps2_io;
mod ps2;

use vga::Color;
use keyboard::{Keyboard, Ps2Keyboard};

const FLOWER: &'static str = include_str!("flower.txt");
const FLOWER_STEM: &'static str = include_str!("flower_stem.txt");


/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    vga::WRITER.lock().fill_screen(Color::Black);
    println!("Flower kernel boot");
    ps2::PS2.lock().initialize();

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

fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
