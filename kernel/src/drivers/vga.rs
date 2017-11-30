use volatile::Volatile;
use core::{cmp, fmt};
use core::convert::TryFrom;
use core::ptr::Unique;
use core::result::Result;

use util::{self, FromDiscriminator};
use color::Color;
use drivers::terminal::{TerminalWriter, SizedTerminalWriter, Point, TerminalCharacter, TerminalColor};

pub const RESOLUTION_X: usize = 80;
pub const RESOLUTION_Y: usize = 25;

/// Interface to VGA, allowing write
pub struct VgaWriter {
    buffer: Unique<VgaBuffer>,
}

impl fmt::Debug for VgaWriter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VgaWriter")
    }
}

#[derive(Debug)]
pub enum VgaWriteError {
}

impl VgaWriter {
    pub const fn new() -> Self {
        VgaWriter {
            buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
        }
    }

    fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.as_mut() }
    }

    /// Fills the screen 4 pixels at a time
    pub fn fill_screen(&mut self, fill_colour: Color) {
        let blank = VgaChar {
            color: VgaColor::new(Color::Black, fill_colour),
            character: b' ',
        };

        for row in 0..RESOLUTION_Y {
            for column in 0..RESOLUTION_X {
                self.buffer().set_char(column, row, blank);
            }
        }
    }
}

impl TerminalWriter for VgaWriter {
    type Error = VgaWriteError;

    fn write(&mut self, char: TerminalCharacter) -> Result<(), Self::Error> {
        self.buffer().set_char(0, 0, VgaChar::new(VgaColor::from(char.color), char.character as u8));
        Ok(())
    }
}

impl SizedTerminalWriter for VgaWriter {
    fn set(&mut self, point: Point, char: TerminalCharacter) -> Result<(), Self::Error> {
        self.buffer().set_char(point.x, point.y, VgaChar::new(VgaColor::from(char.color), char.character as u8));
        Ok(())
    }
}

/// Represents the complete VGA character buffer, containing a 2D array of VgaChar
struct VgaBuffer([[Volatile<VgaChar>; RESOLUTION_X]; RESOLUTION_Y]);

impl VgaBuffer {
    pub fn set_char(&mut self, x: usize, y: usize, value: VgaChar) {
        self.0[y][x].write(value);
    }

    #[allow(dead_code)] // Part of API
    pub fn get_char(&mut self, x: usize, y: usize) -> VgaChar {
        self.0[y][x].read()
    }

    pub fn scroll_down(&mut self, amount: usize, background_color: Color) {
        let amount = cmp::min(amount, RESOLUTION_Y);

        // Shift elements left by amount only if amount < Y resolution
        // If amount is any more then the data will be cleared anyway
        if amount != RESOLUTION_Y {
            self.0.rotate_left(amount);
        }

        // Clear rows up to the amount
        for row in 0..amount {
            self.clear_row((RESOLUTION_Y - 1) - row, background_color);
        }
    }

    pub fn clear_row(&mut self, y: usize, color: Color) {
        let blank = VgaChar::new(
            VgaColor::new(Color::Black, color),
            b' '
        );

        for x in 0..RESOLUTION_X {
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
    fn new(color: VgaColor, character: u8) -> Self {
        VgaChar {
            color: color,
            character: character
        }
    }
}

/// Represents a VGA colour, with both a foreground and background
#[derive(Clone, Copy)]
pub struct VgaColor(u8);

impl VgaColor {
    /// Creates a new VgaColor for the given foreground and background
    pub const fn new(foreground: Color, background: Color) -> VgaColor {
        VgaColor((background as u8) << 4 | (foreground as u8))
    }
}

/// Converts a [TerminalColor] to a [VgaColor] to be displayed
impl From<TerminalColor> for VgaColor {
    fn from(color: TerminalColor) -> Self {
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
