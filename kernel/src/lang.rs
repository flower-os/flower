//! Lang items

use ::halt;
use color::{Color, ColorPair};
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use drivers::vga::VgaWriter;
use spin::RwLock;
use terminal::{Stdout, TerminalOutput};

#[lang = "eh_personality"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn eh_personality() {}

#[panic_implementation]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
// TODO backtrace
extern fn panic_fmt(info: &PanicInfo) -> ! {
    let vga_writer = RwLock::new(VgaWriter::new());
    let mut writer = Stdout(&vga_writer);

    // Ignore the errors because we can't afford to panic in the panic handler

    let _ = writer.set_color(ColorPair::new(Color::Red, Color::Black));

    let arguments = match info.message() {
        Some(args) => *args,
        None => format_args!("undefined"),
    };

    if let Some(loc) = info.location() {
        let _ = write!(&mut writer, "Panicked at \"{}\", {file}:{line}\n", arguments, file = loc.file(), line = loc.line());
    } else {
        let _ = write!(&mut writer, "Panicked at \"{}\" at an undefined location", arguments);
    }

    halt()
}

#[lang = "oom"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn oom() -> ! {
    panic!("Ran out of kernel heap memory!");
}
