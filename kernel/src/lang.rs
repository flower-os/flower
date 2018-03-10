//! Lang items

use core::fmt::{self, Write};
use spin::RwLock;
use drivers::vga::VgaWriter;
use terminal::{self, TerminalOutput, Stdout};
use color::{Color, ColorPair};
use ::halt;

#[lang = "eh_personality"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
// TODO backtrace
extern fn panic_fmt(args: fmt::Arguments, file: &'static str, line: u32) -> ! {
    let vga_writer = RwLock::new(VgaWriter::new());
    let mut writer = Stdout(&vga_writer);

    // Ignore the errors because we can't afford to panic in the panic handler

    let _ = writer.set_cursor_pos(terminal::Point::new(0, 5)); // TODO no
    let _ = writer.set_color(ColorPair::new(Color::Red, Color::Black));
    let _ = write!(&mut writer, "Panicked at \"{}\", {file}:{line}\n", args, file=file, line=line);

    unsafe { halt() }
}
