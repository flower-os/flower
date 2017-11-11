use volatile::Volatile;
use spin::Mutex;
use core::ptr::{Unique, write_volatile};
use core::ops::{Index, IndexMut};
use core::cmp;
use core::convert::{TryFrom, TryInto};

const VGA_ADDR: usize = 0xb8000;
const RESOLUTION_X: usize = 80;
const RESOLUTION_Y: usize = 25;

pub static WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new(
    VgaColor::new(Color::White, Color::Black)
));


/// Interface to VGA, allowing write
pub struct VgaWriter {
    column_position: usize,
    color: VgaColor,
    buffer: Unique<VgaBuffer>,
}

#[derive(Debug)]
pub enum VgaWriteError {
    ColorCodeOutOfBounds(u8)
}

impl VgaWriter {
    const fn new(color: VgaColor) -> Self {
        VgaWriter {
            column_position: 0,
            color: color,
            buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
        }
    }

    pub fn write_char(&mut self, character: char) -> Result<(), VgaWriteError> {
        match character {
            '\n' => self.new_line()?,
            character => {
                if self.column_position >= RESOLUTION_X {
                    self.new_line()?;
                }
                let row = RESOLUTION_Y - 1;
                let column = self.column_position;
                let char_color = self.color;
                self.buffer()[row][column].write(VgaChar {
                    color: char_color,
                    character: character,
                });
                self.column_position += 1;
            }
        }

        Ok(())
    }

    pub fn write_str(&mut self, str: &str) -> Result<(), VgaWriteError> {
        for char in str.chars() {
            self.write_char(char)?;
        }

        Ok(())
    }

    fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) -> Result<(), VgaWriteError> {
        let color = self.color;

        if self.column_position < RESOLUTION_Y - 1 {
            self.column_position += 1
        } else {
            // Scroll down 1
            self.buffer().scroll_down(1,
                                      (color
                                          .try_into()
                                          .map_err(|e: ColorCodeOutOfBounds|
                                              VgaWriteError::ColorCodeOutOfBounds(e.0)
                                          )?: (Color, Color)).1
            );
        }

        Ok(())
    }

    // TODO clear last row only?


    /// Fills the screen 4 pixels at a time
    pub fn fill_screen(&mut self, fill_colour: Color) {
        let blank_char = (fill_colour as u64) << 8;

        let blank = (blank_char << 48) |
            (blank_char << 32) |
            (blank_char << 16) |
            blank_char;

        let vga_ptr = VGA_ADDR as *mut u64;

        // Clear pixels four at a time
        for four_pixels in 0..((RESOLUTION_Y * RESOLUTION_X) / 4) as isize {
            unsafe {
                write_volatile(vga_ptr.offset(four_pixels), blank);
            }
        }
    }
}

/// Represents the complete VGA character buffer, containing a 2D array of VgaChar
struct VgaBuffer([[Volatile<VgaChar>; RESOLUTION_X]; RESOLUTION_Y]);

impl VgaBuffer {
    pub fn scroll_down(&mut self, amount: usize, background_color: Color) {
        let amount = cmp::min(amount, RESOLUTION_Y);

        // Shift elements left by amount only if amount < Y resolution
        // If amount is any more then the data will be cleared anyway
        if amount != RESOLUTION_Y {
            self.0.rotate(amount);
        }

        // Clear rows up to the amount
        for row in 0..amount {
            self.clear_row(row, background_color);
        }
    }

    fn clear_row(&mut self, y: usize, color: Color) {
        let blank = VgaChar::new(
            VgaColor::new(Color::Black, color),
            ' '
        );

        for column in 0..RESOLUTION_X {
            self.0[y][column].write(blank);
        }
    }
}

impl Index<usize> for VgaBuffer {
    type Output = [Volatile<VgaChar>; RESOLUTION_X];

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for VgaBuffer {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}
/// Represents a full character in the VGA buffer, with a character code, foreground and background
#[derive(Copy, Clone)]
#[repr(C)]
struct VgaChar {
    pub color: VgaColor,
    pub character: char,
}

impl VgaChar {
    fn new(color: VgaColor, character: char) -> Self {
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

impl TryFrom<VgaColor> for (Color, Color) {
    type Error = ColorCodeOutOfBounds;

    fn try_from(color: VgaColor) -> Result<Self, Self::Error> {
        Ok((Color::try_from(color.0 >> 4)?, Color::try_from(color.0 & 0x00FF)?))
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