#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(try_from)]
#![feature(nll)]
#![feature(range_contains)]
#![feature(type_ascription)]
#![feature(ptr_internals, align_offset)]
#![feature(arbitrary_self_types)]
#![feature(alloc, allocator_api, box_syntax)]
#![feature(abi_x86_interrupt)]
#![feature(compiler_builtins_lib)]
#![feature(panic_implementation)]
#![feature(panic_info_message)]

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;

extern crate rlibc;
#[cfg_attr(not(test), macro_use)]
extern crate alloc;
extern crate volatile;
extern crate log as log_facade;
extern crate acpi;
extern crate spin;
extern crate x86_64;
extern crate array_init; // Used as a workaround until const-generics arrives
extern crate multiboot2;
extern crate bit_field;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;
extern crate arrayvec;

use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::keyboard::keymap;
use drivers::ps2;
use terminal::TerminalOutput;

#[cfg(not(test))]
mod lang;
#[macro_use]
mod util;
#[macro_use]
mod color;
#[macro_use]
mod log;
#[macro_use]
mod terminal;
mod io;
mod interrupts;
mod memory;
mod drivers;
mod acpi_impl;

use memory::heap::Heap;

#[cfg_attr(not(test), global_allocator)]
pub static HEAP: Heap = Heap::new();

/// Kernel main function
#[no_mangle]
pub extern fn kmain(multiboot_info_addr: usize, guard_page_addr: usize) -> ! {
    say_hello();
    log::init();
    interrupts::init();
    let mb_info = unsafe { multiboot2::load(multiboot_info_addr) };
    memory::init_memory(&mb_info, guard_page_addr);

    // TODO
    use alloc::collections::BTreeMap;
    use alloc::prelude::*;
    let mut m: BTreeMap<String, u32> = BTreeMap::new();
    m.insert("/r/greentext".to_string(), 10);
    let gt = "/r/greentext".to_string();
    trace!("{:?}", "/r/greentext".to_string() == "/r/greentext".to_string());
    trace!("{}", m[&gt]);
    drop(m);

    acpi_impl::acpi_init();

    // Initialize the PS/2 controller and run the keyboard echo loop
    let mut controller = ps2::CONTROLLER.lock();
    match controller.initialize() {
        Ok(_) => info!("ps2c: init successful"),
        Err(error) => error!("ps2c: {:?}", error),
    }

    keyboard_echo_loop(&mut controller);

    halt()
}

/// Say hello to the user and print flower
fn say_hello() {
    terminal::STDOUT.write().clear().expect("Screen clear failed");

    print_flower().expect("Flower print failed");

    terminal::STDOUT.write().set_color(color!(Green on Black))
        .expect("Color should be supported");

    // Print boot message
    println!("Flower kernel boot!");
    println!("-------------------");

    // Reset colors
    terminal::STDOUT.write().set_color(color!(White on Black))
        .expect("Color should be supported");
}

fn print_flower() -> Result<(), terminal::TerminalOutputError<()>> {
    const FLOWER: &'static str = include_str!("resources/art/flower.txt");
    const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");

    let mut stdout = terminal::STDOUT.write();
    let old = stdout.cursor_pos();

    stdout.write_string_colored(FLOWER, color!(LightBlue on Black))?;
    stdout.write_string_colored(FLOWER_STEM, color!(Green on Black))?;
    stdout.set_cursor_pos(old)
}

fn keyboard_echo_loop(controller: &mut ps2::Controller) {
    let keyboard_device = controller.device(ps2::DevicePort::Keyboard);
    let mut keyboard = Ps2Keyboard::new(keyboard_device);
    if let Ok(_) = keyboard.enable() {
        info!("kbd: successfully enabled");
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
        error!("kbd: enable unsuccessful");
    }
}

fn halt() -> ! {
    unsafe {
        // Disable interrupts
        asm!("cli");

        // Halt forever...
        loop {
            asm!("hlt");
        }
    }
}
