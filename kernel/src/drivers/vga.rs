use volatile::Volatile;
use spin::Mutex;
use core::{cmp, fmt};
use core::ptr::Unique;
use core::convert::{TryFrom, TryInto};

const RESOLUTION_X: usize = 80;
const RESOLUTION_Y: usize = 25;

pub static WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new(
    VgaColor::new(Color::White, Color::Black)
));


/// Interface to VGA, allowing write
pub struct VgaWriter {
    column_position: usize,
    row_position: usize,
    color: VgaColor,
    buffer: Unique<VgaBuffer>,
}

#[derive(Debug)]
pub enum VgaWriteError {
    ColorCodeOutOfBounds(u8)
}

#[allow(dead_code)] // For api -- may be used later
impl VgaWriter {
    const fn new(color: VgaColor) -> Self {
        VgaWriter {
            column_position: 0,
            row_position: 0,
            color: color,
            buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
        }
    }

    pub fn set_color(&mut self, color: VgaColor) {
        self.color = color;
    }

    pub fn write_char(&mut self, character: char) -> Result<(), VgaWriteError> {
        let color = self.color;
        self.write_char_colored(character, color)?;

        Ok(())
    }

    pub fn write_char_colored(&mut self, character: char, char_color: VgaColor) -> Result<(), VgaWriteError> {
        match character {
            '\n' => self.new_line()?,
            '\x08' => self.backspace_char()?,
            character => {
                if self.column_position >= RESOLUTION_X {
                    self.new_line()?;
                }
                let row = self.row_position;
                let column = self.column_position;
                self.buffer().set_char(row, column, VgaChar {
                    color: char_color,
                    character: character as u8,
                });
                self.column_position += 1;
            }
        }

        Ok(())
    }

    /// Backspaces one char
    pub fn backspace_char(&mut self) -> Result<(), VgaWriteError> {
        if self.column_position > 0 {
            self.column_position -= 1;
        } else if self.row_position > 0 {
            self.column_position = RESOLUTION_X - 1;
            self.row_position -= 1;
        } else {
            return Ok(());
        }

        let row = self.row_position;
        let column = self.column_position;
        self.set_char(row, column, ' ');

        Ok(())
    }

    pub fn set_char(&mut self, row: usize, column: usize, character: char) {
        let char_color = self.color;
        self.set_char_colored(row, column, character, char_color);
    }

    pub fn set_char_colored(&mut self, row: usize, column: usize, character: char, char_color: VgaColor) {
        self.buffer().set_char(row, column, VgaChar {
            color: char_color,
            character: character as u8,
        });
    }

    pub fn write_str(&mut self, str: &str) -> Result<(), VgaWriteError> {
        let color = self.color;
        self.write_str_colored(str, color)
    }

    pub fn write_str_colored(&mut self, str: &str, color: VgaColor) -> Result<(), VgaWriteError> {
       for char in str.chars() {
            self.write_char_colored(char, color)?;
        }

        Ok(())
    }

    fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) -> Result<(), VgaWriteError> {
        self.column_position = 0;

        if self.row_position < RESOLUTION_Y - 1 {
            self.row_position += 1
        } else {
            // Scroll down 1
            let background_color = self.background_color()
                .map_err(|e| VgaWriteError::ColorCodeOutOfBounds(e.0))?;
            self.buffer().scroll_down(1, background_color);
        }

        Ok(())
    }

    /// Fills the screen 4 pixels at a time
    pub fn fill_screen(&mut self, fill_colour: Color) {
        let blank = VgaChar {
            color: VgaColor::new(Color::Black, fill_colour),
            character: ' ' as u8,
        };

        for row in 0..RESOLUTION_Y {
            for column in 0..RESOLUTION_X {
                self.buffer().set_char(row, column, blank);
            }
        }
    }

    /// Gets the background color for this writer
    pub fn background_color(&mut self) -> Result<Color, ColorCodeOutOfBounds> {
        Ok((self.color.try_into()?: (Color, Color)).0)
    }

    /// Gets the foreground color for this writer
    pub fn foreground_color(&mut self) -> Result<Color, ColorCodeOutOfBounds> {
        Ok((self.color.try_into()?: (Color, Color)).1)
    }

    /// Gets current pos of cursor
    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.row_position, self.column_position)
    }

    /// Sets the current pos of cursor
    pub fn set_cursor_pos(&mut self, pos: (usize, usize)) {
        self.row_position = pos.0;
        self.column_position = pos.1;
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, str: &str) -> Result<(), fmt::Error> {
        self.write_str(str).map_err(|_| fmt::Error)
    }
}

/// Represents the complete VGA character buffer, containing a 2D array of VgaChar
struct VgaBuffer([[Volatile<VgaChar>; RESOLUTION_X]; RESOLUTION_Y]);

#[allow(dead_code)]
impl VgaBuffer {
    pub fn set_char(&mut self, x: usize, y: usize, value: VgaChar) {
        self.0[x][y].write(value);
    }

    pub fn get_char(&mut self, x: usize, y: usize) -> VgaChar {
        self.0[x][y].read()
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
            ' '
        );

        for column in 0..RESOLUTION_X {
            self.0[y][column].write(blank);
        }
    }
}

/// Represents a full character in the VGA buffer, with a character code, foreground and background
#[derive(Copy, Clone)]
#[repr(C)]
struct VgaChar {
    pub character: u8,
    pub color: VgaColor,
}

impl VgaChar {
    fn new(color: VgaColor, character: char) -> Self {
        VgaChar {
            color: color,
            character: character as u8
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

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::drivers::vga::stdout_print(format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn stdout_print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
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
