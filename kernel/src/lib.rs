#![no_std]

#![feature(asm)]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(const_unique_new)]
#![feature(unique)]

extern crate rlibc;

mod lang;
mod vga;

use core::ptr::write_volatile;

/// Kernel main function
#[no_mangle]
pub extern fn kmain() -> ! {
    clear_screen();
    boot_print();

    unsafe { asm!("hlt"); }
    loop {}
}

// TODO could be optimized to clear 4 pixels at a time
fn clear_screen() {

    // TODO could be constants
    let bottom_right_pixel = (25 * 80) + 80; // Pixel with highest value
    let vga_ptr = 0xb8000 as *mut u16;

    for location in 0..bottom_right_pixel {
        unsafe {
            write_volatile(vga_ptr.offset(location), 0)
        }
    }

}

fn boot_print() {

    // TODO could be constant
    let vga_ptr = 0xb8000 as *mut u16;

    for (index, char) in b"Flower kernel boot".iter().enumerate() {
        unsafe {
            write_volatile(vga_ptr.offset(index as isize), 0x0200u16 | *char as u16)
        }
    }

}
