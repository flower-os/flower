use core::ptr::Unique;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

/// Interface to VGA, allowing write
pub struct VgaWriter {
    column_position: usize,
    color: VgaColor,
    buffer: Unique<VgaBuffer>,
}

impl VgaWriter {
    pub fn write_char(&mut self, character: char) {
        match character {
            '\n' => self.new_line(),
            character => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let column = self.column_position;
                let char_color = self.color;
                self.buffer().0[row][column] = VgaChar {
                    color: char_color,
                    character: character,
                };
                self.column_position += 1;
            }
        }
    }

    fn buffer(&mut self) -> &mut VgaBuffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) {}
}

/// Represents the complete VGA character buffer, containing a 2D array of VgaChar
struct VgaBuffer([[VgaChar; BUFFER_WIDTH]; BUFFER_HEIGHT]);

/// Represents a full character in the VGA buffer, with a character code, foreground and background
#[repr(C)]
struct VgaChar {
    color: VgaColor,
    character: char,
}

/// Represents a VGA colour, with both a foreground and background
#[derive(Debug, Clone, Copy)]
struct VgaColor(u8);

impl VgaColor {
    /// Creates a new VgaColor for the given foreground and background
    const fn new_color(foreground: Color, background: Color) -> VgaColor {
        VgaColor((background as u8) << 4 | (foreground as u8))
    }
}

/// Represents valid VGA colors
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
