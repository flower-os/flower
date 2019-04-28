#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(nll)]
#![feature(type_ascription)]
#![feature(ptr_internals, align_offset)]
#![feature(arbitrary_self_types)]
#![feature(allocator_api, box_syntax)]
#![feature(abi_x86_interrupt)]
#![feature(compiler_builtins_lib)]
#![feature(panic_info_message)]
#![feature(integer_atomics)]
#![feature(try_trait)]

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
extern crate array_init;
// Used as a workaround until const-generics arrives
extern crate multiboot2;
extern crate bit_field;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate static_assertions;
extern crate arrayvec;
extern crate crossbeam;

//use crate::drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
//use crate::drivers::keyboard::keymap;
use crate::drivers::{ps2, serial};
use crate::terminal::TerminalOutput;

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
mod gdt;
mod cpuid;
mod snake;

use crate::memory::heap::Heap;
use crate::drivers::keyboard::{keymap, Ps2Keyboard, Keyboard, KeyEventType};

#[cfg_attr(not(test), global_allocator)]
pub static HEAP: Heap = Heap::new();

/// Kernel main function
#[no_mangle]
pub extern fn kmain(multiboot_info_addr: usize, guard_page_addr: usize) -> ! {
    serial::PORT_1.lock().init(serial::MAX_BAUD, false).expect("Error initializing serial port 1");
    say_hello();
    info!("serial: initialized port 1");
    log::init();
    memory::init_memory(multiboot_info_addr, guard_page_addr);
    gdt::init();
    interrupts::init();
    interrupts::enable();
    info!("interrupts: ready");

    drivers::pit::CONTROLLER.lock().initialize();

    let _acpi = acpi_impl::acpi_init();

    // Initialize the PS/2 controller and run the keyboard echo loop
    match ps2::initialize() {
        Ok(_) => info!("ps2c: init successful"),
        Err(error) => error!("ps2c: {:?}", error),
    }

    snake::snake();

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

    serial_println!("Flower kernel boot!");
    serial_println!("-------------------");

    // Reset colors
    terminal::STDOUT.write().set_color(color!(White on Black))
        .expect("Color should be supported");
}

fn print_flower() -> Result<(), terminal::TerminalOutputError<()>> {
    const FLOWER: &'static str = include_str!("resources/art/flower.txt");
    const FLOWER_STEM: &'static str = include_str!("resources/art/flower_stem.txt");

    let mut stdout = terminal::STDOUT.write();
    let old = stdout.cursor_pos().expect("Terminal must support cursor");

    stdout.write_string_colored(FLOWER, color!(LightBlue on Black))?;
    stdout.write_string_colored(FLOWER_STEM, color!(Green on Black))?;
    stdout.set_cursor_pos(old)?;

    serial_print!("{}", FLOWER);
    serial_println!("{}", FLOWER_STEM);

    Ok(())
}

fn keyboard_echo_loop() {
    let mut keyboard = Ps2Keyboard::new();
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
}

fn halt() -> ! {
    unsafe {
        // Disable interrupts
        asm!("cli" :::: "volatile");

        // Halt forever...
        loop {
            asm!("hlt" :::: "volatile");
        }
    }
}
