#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique, const_unique_new)]
#![feature(slice_rotate)]
#![feature(try_from)]
#![feature(nll)]
#![feature(inclusive_range_syntax)]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate x86_64;
extern crate array_init; // Used as a workaround until const-generics arrives

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

mod lang;
#[macro_use]
mod util;
#[macro_use]
mod color;
mod io;
#[macro_use]
mod terminal;
mod drivers;

use terminal::{TerminalOutput, TerminalCharacter};
use drivers::ps2;
use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::keyboard::keymap;
use drivers::vga;

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {

    terminal::STDOUT.write().clear().expect("Screen clear failed");

    print_flower().expect("Flower print failed");

    terminal::STDOUT.write().set_color(color!(Green on Black))
        .expect("Color should be supported");

    // Print boot message
    println!("Flower kernel boot!");
    println!("-------------------\n");

    // Reset colors
    terminal::STDOUT.write().set_color(color!(White on Black))
        .expect("Color should be supported");

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
                    if event.keycode == keymap::codes::BACKSPACE {
                        // Ignore error
                        let _ = terminal::STDOUT.write().backspace();
                    } else if let Some(character) = event.char {
                        print!("{}", character)
                    }
                }
            }
        }
    } else {
        println!("kbd: enable unsuccessful");
    }

    unsafe { halt() }
}

fn print_flower() -> Result<(), terminal::TerminalOutputError<()>> {
    const FLOWER: &'static str = include_str!("resources/art/flower.txt");
    const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");
    const SIZE: terminal::Resolution = terminal::Resolution::new(47, 25);

    let mut stdout = terminal::STDOUT.write();
    let old = stdout.cursor_pos();

    stdout.write_string_colored(FLOWER, color!(LightBlue on Black))?;
    stdout.write_string_colored(FLOWER_STEM, color!(Green on Black))?;
    stdout.set_cursor_pos(old)
}

unsafe fn halt() -> ! {
    asm!("cli");

    loop {
        asm!("hlt")
    }
}