//! Lang items

use ::halt;
use color::{Color, ColorPair};
use core::{alloc::Layout, panic::PanicInfo, fmt::Write};
use drivers::vga::VgaWriter;
use spin::RwLock;
use terminal::{Stdout, TerminalOutput};

// A note on the `#[no_mangle]`s:
// Apparently, removing them makes it link-error with undefined symbols, so we include them

#[lang = "eh_personality"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn eh_personality() {}

#[panic_implementation]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn panic_implementation(panic_info: &PanicInfo) -> ! {
    let vga_writer = RwLock::new(VgaWriter::new());
    let mut writer = Stdout(&vga_writer);

    // Ignore the errors because we can't afford to panic in the panic handler
    let _ = writer.set_color(ColorPair::new(Color::Red, Color::Black));
    let _ = write!(&mut writer, "Kernel {}", panic_info);

    halt()
}

#[lang = "oom"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn oom(layout: Layout) -> ! {
    panic!("Ran out of kernel heap memory allocating {:?}", layout);
}
