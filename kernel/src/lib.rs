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
#[macro_use]
mod util;
mod io;
mod color;
#[macro_use]
mod terminal;
mod drivers;

use drivers::vga::{self, VgaWriter};
use color::Color;
use terminal::{STDOUT, Terminal, TextArea, Point, TerminalColor, TerminalWriteError};

const FLOWER: &'static str = include_str!("resources/art/flower.txt");
const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    terminal::WRITER.lock().fill_screen(Color::Black);

    print_flower().expect("Flower should be printed");

    STDOUT.lock().set_color(TerminalColor::new(Color::Green, Color::Black))
        .expect("Color should be set");

    // Print boot message
    println!("Flower kernel boot!");
    println!("-------------------\n");

    // Reset colors
    STDOUT.lock().set_color(TerminalColor::new(Color::White, Color::Black))
        .expect("Color should be set");

    drivers::ps2::PS2.lock().initialize();

    halt()
}

fn print_flower() -> Result<(), TerminalWriteError<VgaWriter>> {
    let mut area = TextArea::new(
        &terminal::WRITER,
        Point::new(32, 0),
        Point::new(vga::RESOLUTION_X - 32, vga::RESOLUTION_Y)
    );

    area.set_color(TerminalColor::new(Color::LightBlue, Color::Black))?;
    area.write_string("\n")?;
    area.write_string(FLOWER)?;
    area.set_color(TerminalColor::new(Color::Green, Color::Black))?;
    area.write_string(FLOWER_STEM)?;

    Ok(())
}

// TODO move somewhere else
fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
