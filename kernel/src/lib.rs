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
mod color;
#[macro_use]
mod drivers;

use spin::Mutex;
use drivers::vga::{self, VgaWriter};
use color::Color;
use drivers::terminal::{self, STDOUT, Terminal, TextArea, Point, TerminalColor, TerminalWriteError};

pub static FLOWER_AREA: Mutex<TextArea<VgaWriter>> = Mutex::new(TextArea::new(&terminal::WRITER, Point::new(32, 0), Point::new(vga::RESOLUTION_X - 32, vga::RESOLUTION_Y)));

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
    println!("-------------------");
    println!("");

    // Reset colors
    STDOUT.lock().set_color(TerminalColor::new(Color::White, Color::Black))
        .expect("Color should be set");

    drivers::ps2::PS2.lock().initialize();

    halt()
}

fn print_flower() -> Result<(), TerminalWriteError<VgaWriter>> {
    let mut area = FLOWER_AREA.lock();
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
