use volatile::Volatile;
use core::cmp;
use core::ptr::Unique;
use core::convert::TryFrom;
use core::result::Result;

use drivers::terminal::{TerminalWriter, TerminalColor};

pub const RESOLUTION_X: usize = 80;
pub const RESOLUTION_Y: usize = 25;

/// Interface to VGA, allowing write
pub struct VgaWriter {
    buffer: Unique<VgaBuffer>,
}

#[derive(Debug)]
pub enum VgaWriteError {
}

#[allow(dead_code)] // For api -- may be used later
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

    fn set(&mut self, column: usize, row: usize, character: u8, color: TerminalColor) -> Result<(), VgaWriteError> {
        self.buffer().set_char(column, row, VgaChar::new(VgaColor::new(color.foreground, color.background), character));

        Ok(())
    }
}

/// Represents the complete VGA character buffer, containing a 2D array of VgaChar
struct VgaBuffer([[Volatile<VgaChar>; RESOLUTION_X]; RESOLUTION_Y]);

#[allow(dead_code)]
impl VgaBuffer {
    pub fn set_char(&mut self, x: usize, y: usize, value: VgaChar) {
        self.0[y][x].write(value);
    }

    pub fn get_char(&mut self, x: usize, y: usize) -> VgaChar {
        self.0[y][x].read()
    }

    pub fn scroll_down(&mut self, amount: usize, background_color: Color) {
        let amount = cmp::min(amount, RESOLUTION_Y);

        // Shift elements left by amount only if amount < Y resolution
        // If amount is any more then the data will be cleared anyway
        if amount != RESOLUTION_Y {
            self.0.rotate(amount);
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

/// Converts VgaColor to tuple of `(background, foreground)`
impl TryFrom<VgaColor> for (Color, Color) {
    type Error = ColorCodeOutOfBounds;

    fn try_from(color: VgaColor) -> Result<Self, Self::Error> {
        Ok((Color::try_from((color.0 & 0xF0) >> 4)?, Color::try_from(color.0 & 0x0F)?))
    }
}

/// Represents valid VGA colors
#[allow(dead_code)] // dead variants for completeness
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Struct to show that the color code was out of bounds for [TryFrom] for [Color]
pub struct ColorCodeOutOfBounds(u8);

impl TryFrom<u8> for Color {
    /// The only type of error is out of bounds
    type Error = ColorCodeOutOfBounds;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Color::*;
        match value {
            0 => Ok(Black),
            1 => Ok(Blue),
            2 => Ok(Green),
            3 => Ok(Cyan),
            4 => Ok(Red),
            5 => Ok(Magenta),
            6 => Ok(Brown),
            7 => Ok(LightGray),
            8 => Ok(DarkGray),
            9 => Ok(LightBlue),
            10 => Ok(LightGreen),
            11 => Ok(LightCyan),
            12 => Ok(LightRed),
            13 => Ok(Pink),
            14 => Ok(Yellow),
            15 => Ok(White),
            code => Err(ColorCodeOutOfBounds(code))
        }
    }
}
