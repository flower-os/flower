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
mod io;
#[macro_use]
mod drivers;

use drivers::ps2;
use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::vga::{self, VgaColor, Color};

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

    halt()
}

// TODO move somewhere else
fn halt() -> ! {
    unsafe { asm!("hlt") }
    loop {} // Required to trick rust
}
