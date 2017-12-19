#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique, const_unique_new)]
#![feature(slice_rotate)]
#![feature(try_from)]

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

use terminal::{TerminalOutput, TerminalCharacter, Resolution};
use util::halt;
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
    terminal::STDOUT.lock().clear().expect("Screen clear failed");

    // TODO print first
    print_flower().expect("Flower print failed");

    terminal::STDOUT.lock().set_color(color!(Green on Black))
        .expect("Color should be supported");

    // Print boot message
    // TODO own text area
    println!("Flower kernel boot!");
    println!("-------------------\n");

    // Reset colors
    terminal::STDOUT.lock().set_color(color!(White on Black))
        .expect("Color should be supported");

    let mut controller = ps2::CONTROLLER.lock();
    match controller.initialize() {
        Ok(_) => println!("ps2c: successful initialization"),
        Err(error) => println!("ps2c: threw error: {:?}", error),
    }

    // TODO print first
    print_flower().expect("Flower print failed");

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

fn print_flower() -> Result<(), terminal::TerminalOutputError<()>> {
    const FLOWER: &'static str = include_str!("resources/art/flower.txt");
    const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");
    const SIZE: Resolution = Resolution::new(vga::RESOLUTION.x - 32, vga::RESOLUTION.y);

    // ~*~* Entering unsafe hack... *~*~
    // Unsafe magic to make the array of arrays into an array of slices
    // Thanks to matklad (github usr) and nvzqz (github usr)
    // This whole solution in general is just one giant hack until we get const generics

    let mut array_of_arrays = array_init::array_init::<[_; SIZE.y], _>(
        |_| [TerminalCharacter::new(' ', color!(Black on Black)); SIZE.x]
    );

    let mut buf: [&mut [TerminalCharacter]; SIZE.y] = unsafe { core::mem::uninitialized() };

    for (slot, array) in buf.iter_mut().zip(array_of_arrays.iter_mut()) {
        unsafe { core::ptr::write(slot, array); }
    }

    // ~*~* Exiting unsafe hack... *~*~

    let mut area = terminal::TextArea::new(
        &vga::WRITER,
        terminal::Point::new(32, 0),
        SIZE,
        &mut buf
    );

    area.write_string("Hello, TextAreas!\nA")?;
    area.write_string("Hello, TextAreas 2!")?;
    area.set_color(color!(LightBlue on Black))?;
    area.write('\n')?;
    area.write_string(FLOWER)?;
    area.set_color(color!(Green on Black))?;
    area.write_string(FLOWER_STEM)
}

unsafe fn halt() -> ! {
    asm!("cli");

    loop {
        asm!("hlt")
    }
}