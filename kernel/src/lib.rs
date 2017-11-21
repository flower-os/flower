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

use spin::Mutex;
use drivers::vga::{self, VgaWriter, Color};
use drivers::terminal::{self, TerminalColor};
use drivers::terminal::text_area::{STDOUT, TextArea, AreaWriter};

pub static FLOWER_AREA: Mutex<AreaWriter<VgaWriter>> = Mutex::new(AreaWriter::new(32, 0, vga::RESOLUTION_X - 32, vga::RESOLUTION_Y, &terminal::WRITER));

const FLOWER: &'static str = include_str!("resources/art/flower.txt");
const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    terminal::WRITER.lock().fill_screen(Color::Black);

    // Print flower
    FLOWER_AREA.lock().set_color(TerminalColor::new(Color::LightBlue, Color::Black));
    FLOWER_AREA.lock().write_string("\n");
    FLOWER_AREA.lock().write_string(FLOWER);
    FLOWER_AREA.lock().set_color(TerminalColor::new(Color::Green, Color::Black));
    FLOWER_AREA.lock().write_string(FLOWER_STEM);

    STDOUT.lock().set_color(TerminalColor::new(Color::Green, Color::Black));

    // Print boot message
    println!("Flower kernel boot!");
    println!("-------------------");
    println!("");

    // Reset colors
    STDOUT.lock().set_color(TerminalColor::new(Color::White, Color::Black));

    drivers::ps2::PS2.lock().initialize();

    halt()
}

// TODO move somewhere else
fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
