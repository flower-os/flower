use volatile::Volatile;
use core::{cmp, fmt};
use core::convert::TryFrom;
use core::ptr::Unique;
use core::result::Result;
use spin::Mutex;

use util::{self, FromDiscriminator};
use color::{Color, ColorPair};
use terminal::*;

pub static WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new());

/// The resolution of VGA
pub const RESOLUTION: Resolution = Resolution::new(80, 25);

/// Interface to VGA, allowing write
pub struct VgaWriter {
    buffer: Unique<VgaBuffer>,
    cursor: Point,
    color: ColorPair,
}

impl fmt::Debug for VgaWriter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VgaWriter")
    }
}

impl VgaWriter {
    pub const fn new() -> Self {
        VgaWriter {
            buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
            cursor: Point::new(0, RESOLUTION.y - 1),
            color: color!(White on Black),
        }
    }

    // TODO nonpublic
    pub fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.as_mut() }
    }
}

impl TerminalOutput<()> for VgaWriter {

    fn resolution(&self) -> Resolution {
        RESOLUTION
    }

    fn color_supported(&self, _color: Color) -> bool {
        true // For now, the color scheme is the vga color scheme
    }

    fn cursor_pos(&self) -> Point {
        self.cursor
    }

    fn set_cursor_pos(&mut self, cursor: Point) -> Result<(), TerminalOutputError<()>> {
        if self.in_bounds(cursor) {
            self.cursor = cursor;
            Ok(())
        } else {
            // TODO
            let x = cursor.x;
            let y = cursor.y;
            Err(TerminalOutputError::OutOfBounds(cursor))
        }
    }

    fn color(&self) -> ColorPair {
        self.color
    }

    fn set_color(&mut self, color: ColorPair) -> Result<(), TerminalOutputError<()>> {
        if !self.color_supported(color.foreground) {
            return Err(TerminalOutputError::ColorUnsupported(color.foreground));
        }
        if !self.color_supported(color.background) {
            return Err(TerminalOutputError::ColorUnsupported(color.background));
        }

        self.color = color;

        Ok(())
    }

    fn set_char(&mut self, char: TerminalCharacter, point: Point) -> Result<(), TerminalOutputError<()>> {

        let point = Point::new(point.x, RESOLUTION.y - 1 - point.y);

        if !self.color_supported(char.color.foreground) {
            return Err(TerminalOutputError::ColorUnsupported(char.color.foreground));
        }
        if !self.color_supported(char.color.background) {
            return Err(TerminalOutputError::ColorUnsupported(char.color.background));
        }

        if !self.in_bounds(point) {
            return Err(TerminalOutputError::OutOfBounds(point));
        }

        self.buffer().set_char(
            point.x,
            point.y,
            VgaChar::new(
                VgaColor::from(char.color),
                char.character as u8
            )
        );
        Ok(())
    }

    fn write_raw(&mut self, char: TerminalCharacter) -> Result<(), TerminalOutputError<()>> {
        let mut pos = self.cursor_pos();
        self.set_char(char, pos)?;
        // TODO something fishy in set_char
        pos.x += 1;

        // If the x point went out of bounds, wrap
        if pos.x >= RESOLUTION.x {
            self.new_line()?;
        } else {
            self.set_cursor_pos(pos)?;
        }

        Ok(())
    }

    fn clear_line(&mut self, y: usize) -> Result<(), TerminalOutputError<()>> {
        let color = self.color().background;

        if self.in_bounds(Point::new(0, y)) {
            self.buffer().clear_row(y, color);
            Ok(())
        } else {
            Err(TerminalOutputError::OutOfBounds(Point::new(0, y)))
        }
    }

    fn clear(&mut self) -> Result<(), TerminalOutputError<()>> {
        for line in 0..self.resolution().y {
            self.clear_line(line)?;
        }

        Ok(())
    }

    fn scroll_down(&mut self, amount: usize) -> Result<(), TerminalOutputError<()>> {
        let background = self.color.background;
        self.buffer().scroll_down(amount, background);

        Ok(())
    }
}

/// Represents the complete VGA character buffer, containing a 2D array of VgaChar
// TODO nonpublic
#[repr(C)]
pub struct VgaBuffer([[Volatile<VgaChar>; RESOLUTION.x]; RESOLUTION.y]);

impl VgaBuffer {
    pub fn set_char(&mut self, x: usize, y: usize, value: VgaChar) {
        self.0[y][x].write(value);
    }

    pub fn scroll_down(&mut self, amount: usize, background_color: Color) {
        let amount = cmp::min(amount, RESOLUTION.y);

        // Shift lines left (up) by amount only if amount < Y resolution
        // If amount is any more then the data will be cleared anyway
        if amount != RESOLUTION_Y {
            self.0.rotate_left(amount);
        }

        // Clear rows up to the amount
        for row in 0..amount {
            self.clear_row((RESOLUTION.y - 1) - row, background_color);
        }
    }

    pub fn clear_row(&mut self, y: usize, color: Color) {
        let blank = VgaChar::new(
            VgaColor::new(Color::Black, color),
            b' '
        );

        for x in 0..RESOLUTION.x {
            self.0[y][x].write(blank);
        }
    }
}

/// Represents a full character in the VGA buffer, with a character code, foreground and background
#[derive(Copy, Clone)]
#[repr(C)]
pub struct VgaChar {
    pub character: u8,
    pub color: VgaColor,
}

impl VgaChar {
    // TODO nonpublic
    pub fn new(color: VgaColor, character: u8) -> Self {
        VgaChar { color, character }
    }
}

/// Represents a VGA colour, with both a foreground and background
#[derive(Clone, Copy)]
#[repr(C)]
pub struct VgaColor(u8);

impl VgaColor {
    /// Creates a new VgaColor for the given foreground and background
    pub const fn new(foreground: Color, background: Color) -> Self {
        VgaColor((background as u8) << 4 | (foreground as u8))
    }
}

/// Converts a [ColorPair] to a [VgaColor] to be displayed
impl From<ColorPair> for VgaColor {
    fn from(color: ColorPair) -> Self {
        VgaColor::new(color.foreground, color.background)
    }
}

/// Converts [VgaColor] to tuple of `(background, foreground)`
impl TryFrom<VgaColor> for (Color, Color) {
    type Error = util::UnknownDiscriminator;

    fn try_from(color: VgaColor) -> Result<Self, Self::Error> {
        Ok((
            Color::from_discriminator(((color.0 & 0xF0) >> 4) as u64)?,
            Color::from_discriminator((color.0 & 0x0F) as u64)?
        ))
    }
}
