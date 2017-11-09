use volatile::Volatile;
use spin::Mutex;
use core::ptr::{Unique, write_volatile};
use core::ops::{Deref, DerefMut};

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

impl VgaWriter {
    const fn new(color: VgaColor) -> Self {
        VgaWriter {
            column_position: 0,
            color: color,
            buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
        }
    }

    pub fn write_char(&mut self, character: char) {
        match character {
            '\n' => self.new_line(),
            character => {
                if self.column_position >= RESOLUTION_X {
                    self.new_line();
                }
                let row = RESOLUTION_Y - 1;
                let column = self.column_position;
                let char_color = self.color;
                self.buffer()[row][column].write(VgaChar {
                    color: char_color,
                    character: character as u8,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_str(&mut self, str: &str) {
        for char in str.chars() {
            self.write_char(char);
        }
    }

    fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) {
        if self.column_position < RESOLUTION_Y - 1 {
            self.column_position += 1
        } else {
            self.buffer().rotate(1); // Rotate 1 (shift elements left 1)
            self.clear_row(RESOLUTION_Y - 1);
        }
    }

    // TODO clear last row only?
    fn clear_row(&mut self, y: usize) {
        let blank = VgaChar::new(self.color, ' ');

        for column in 0..RESOLUTION_X {
            self.buffer()[y][column].write(blank);
        }
    }

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

impl Deref for VgaBuffer {
    type Target = [[Volatile<VgaChar>; RESOLUTION_X]; RESOLUTION_Y];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VgaBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

/// Represents valid VGA colors
#[allow(dead_code)] // dead variants for completeness
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
