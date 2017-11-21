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
#[macro_use]
mod drivers;
mod io;

use spin::Mutex;
use drivers::vga::{self, VgaWriter, Color};
use drivers::terminal::{self, TerminalColor};
use drivers::terminal::text_area::{STDOUT, TextArea, AreaWriter};
use drivers::ps2;
use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::vga::{self, VgaColor, Color};

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
    println!();

    // Reset colors
    STDOUT.lock().set_color(TerminalColor::new(Color::White, Color::Black)
    );

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

unsafe fn halt() -> ! {
    asm!("cli");

    loop {
        asm!("hlt")
    }
}
