#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(unique)]
#![feature(slice_rotate)]
#![feature(try_from)]
#![feature(type_ascription)]
#![feature(range_contains)]
#![feature(iterator_step_by)]
#![feature(use_nested_groups)]
#![feature(ptr_internals)]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate x86_64;
extern crate either;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

use core::convert::TryInto;
use either::{Left, Right};
use drivers::keyboard::{Keyboard, KeyEventType, Ps2Keyboard};
use drivers::ps2;
use drivers::vga::{self, Color, VgaColor};
use acpi::sdt::{SdtHeader, rsdt::TableAddresses};

mod lang;
#[macro_use]
mod util;
#[macro_use]
mod drivers;
mod io;
mod acpi;

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
        VgaColor::new(Color::Green, Color::Black),
    ).expect("Color code should be valid");

    let acpi_res: Result<(), &str> = catch! {
        let (rsdp, address) = match acpi::rsdp::search_for_rsdp() {
            Some(ret) => ret,
            None => return Err("acpi: rsdp not found"),
        };

        println!("acpi: rsdp found at {:#x}", address);

        let mut temp = rsdp
            .map_left(|rsdp_v1| {
                println!("acpi: using rsdt");
                rsdp_v1.try_into().map_err(|_| "acpi: invalid rsdt")
            })
            .map_right(|rsdp_v2| {
                println!("acpi: using xsdt");
                rsdp_v2.try_into().map_err(|_| "acpi: invalid xsdt")
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

        for address in sdt_addresses {
            let header = unsafe { *(address as *const SdtHeader ) };

            print!("    \"");
            for c in header.signature.iter() {
                print!("{}", *c as char);
            }

            println!("\" at {:#x}", address);
        }

        Ok(())
    };

    if let Err(msg) = acpi_res {
        println!("{}", msg);
    }

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
