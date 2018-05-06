#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique, const_unique_new)]
#![feature(slice_rotate)]
#![feature(try_from)]
#![feature(try_trait)]
#![feature(nll)]
#![feature(inclusive_range_syntax)]
#![feature(type_ascription)]
#![feature(range_contains)]
#![feature(iterator_step_by)]
#![feature(use_nested_groups)]
#![feature(ptr_internals)]
#![feature(abi_x86_interrupt)]

extern crate array_init;
#[macro_use]
extern crate bitflags;
extern crate either;
#[macro_use]
extern crate lazy_static;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;
// Used as a workaround until const-generics arrives

use acpi::sdt::{madt::{Madt, MADT_HEADER}, SdtHeader};
use core::{convert::TryInto, ops::Try};
use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::keyboard::keymap;
use drivers::ps2;
use either::{Left, Right};
use terminal::TerminalOutput;

mod lang;
#[macro_use]
mod log;
#[macro_use]
mod util;
#[macro_use]
mod color;
mod io;
mod acpi;
mod interrupts;

#[macro_use]
mod terminal;
mod drivers;

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    interrupts::init();

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
        Ok(_) => info!("ps2c: init successful"),
        Err(error) => error!("ps2c: {:?}", error),
    }

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
        println!("kbd: enable unsuccessful");
    }

    let acpi_res: Result<(), &str> = catch! {
        let (rsdp, address) = match acpi::rsdp::search_for_rsdp() {
            Some(ret) => ret,
            None => return Err("rsdp not found"),
        };

        println!("acpi: rsdp found at {:#x}", address);

        let mut temp = rsdp
            .map_left(|rsdp_v1| {
                println!("acpi: using rsdt");
                rsdp_v1.try_into().map_err(|_| "invalid rsdt")
            })
            .map_right(|rsdp_v2| {
                println!("acpi: using xsdt");
                rsdp_v2.try_into().map_err(|_| "invalid xsdt")
            });

        let (header, sdt_addresses) = match temp {
            Left(Ok((ref mut header, ref mut addresses))) => {
                (header, addresses as &mut Iterator<Item=usize>)
            },
            Right(Ok((ref mut header, ref mut addresses))) => {
                (header, addresses as &mut Iterator<Item=usize>)
            },
            Left(Err(ref mut err)) => return Err(*err),
            Right(Err(ref mut err)) => return Err(*err)
        };

        print!("acpi: oem: \"");
        for c in header.oem_id.iter() {
            print!("{}", *c as char);
        }
        println!("\"");

        println!("acpi: {} tables found", sdt_addresses.size_hint().0);
        println!("acpi: table signatures:");

        let mut madt_address: Option<usize> = None;

        for address in sdt_addresses {
            let header = unsafe { *(address as *const SdtHeader) };

            print!("    \"");
            for c in header.signature.iter() {
                print!("{}", *c as char);
            }

            println!("\" at {:#x}", address);

            if header.signature == *MADT_HEADER {
                madt_address = Some(address);
            }
        }

        let madt_address = madt_address
            .into_result()
            .map_err(|_| "MADT not found")?;

        let madt = unsafe { Madt::from_address(madt_address) }
            .map_err(|_| "Error validating MADT")?;

        println!("acpi: press S for next madt entry");

        for entry in madt.entries {
            loop {
                if let Ok(Some(event)) = keyboard.read_event() {
                    if event.event_type == KeyEventType::Break && event.keycode == keymap::codes::S {
                            break;
                    }
                }
            }

            println!("{:#?}", entry);
        }

        Ok(())
    };

    if let Err(msg) = acpi_res {
        println!("acpi: {}", msg);
    }

    halt()
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
