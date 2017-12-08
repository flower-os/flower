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

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

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
use drivers::ps2;
use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::vga::{self, VgaColor, Color};

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

    let mut controller = ps2::CONTROLLER.lock();
    match controller.initialize() {
        Ok(_) => println!("ps2c: successful initialization"),
        Err(error) => println!("ps2c: threw error: {:?}", error),
    }

    let keyboard_device = controller.device(ps2::DevicePort::Keyboard);
    let mut keyboard = Ps2Keyboard::new(keyboard_device);
    if let Ok(_) = keyboard.enable() {
        println!("kbd: successfully enabled");
        loop {
            if let Ok(Some(event)) = keyboard.read_event() {
                if event.event_type != KeyEventType::Break {
                    if let Some(char) = event.char {
                        print!("{}", char);
                    }
                }
            }
        }
    } else {
        println!("kbd: enable unsuccessful");
    }

    unsafe { halt() }
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

unsafe fn halt() -> ! {
    asm!("cli");

    loop {
        asm!("hlt")
    }
}
