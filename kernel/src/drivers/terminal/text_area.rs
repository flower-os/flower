use core::fmt;
use spin::Mutex;
use drivers::vga::{self, VgaWriter, Color};
use drivers::terminal::{self, TerminalWriter, TerminalColor, TerminalWriteError};

pub static STDOUT: Mutex<AreaWriter<VgaWriter>> = Mutex::new(AreaWriter::new(0, 0, vga::RESOLUTION_X, vga::RESOLUTION_Y, &terminal::WRITER));

/// Represents an area on the screen that text can be written to
pub trait TextArea<T: TerminalWriter> {
    /// Sets the color of this text area
    fn set_color(&mut self, color: TerminalColor);

    /// Writes a string to this area
    fn write_string(&mut self, string: &str) -> Result<(), TerminalWriteError<T::Error>>;

    /// Writes a character to the area with the set color
    fn write_char(&mut self, character: char) -> Result<(), TerminalWriteError<T::Error>>;

    /// Writes a character to the area with color
    fn write(&mut self, character: char, color: TerminalColor) -> Result<(), TerminalWriteError<T::Error>>;

    /// Backspaces the last written character. Used for backspace key presses
    fn backspace(&mut self) -> Result<(), TerminalWriteError<T::Error>>;

    /// Moves the pointer to the next line, and shifts up with exceeded limit
    fn new_line(&mut self) -> Result<(), TerminalWriteError<T::Error>>;
}

/// Represents an area on the screen containing its own set of text
pub struct AreaWriter<'a, T: TerminalWriter + 'a> {
    pub origin_x: usize,
    pub origin_y: usize,
    pub width: usize,
    pub height: usize,
    writer: &'a Mutex<T>,
    pointer_x: usize,
    pointer_y: usize,
    color: TerminalColor,
}

impl<'a, T: TerminalWriter> AreaWriter<'a, T> {
    pub const fn new(origin_x: usize, origin_y: usize, width: usize, height: usize, writer: &'a Mutex<T>) -> Self {
        AreaWriter {
            origin_x: origin_x,
            origin_y: origin_y,
            width: width,
            height: height,
            writer: writer,
            pointer_x: origin_x,
            pointer_y: origin_y,
            color: TerminalColor::new(Color::White, Color::Black),
        }
    }

    /// Writes a single non-special character to the screen
    fn write_single_char(&mut self, character: char, color: TerminalColor) -> Result<(), TerminalWriteError<T::Error>> {
        // Wrap if writing char will exceed screen limits
        if self.pointer_x + 1 >= self.origin_x + self.width {
            self.new_line()?;
        }

        self.writer.lock().set(self.pointer_x, self.pointer_y, character as u8, color)
            .map_err(|e| TerminalWriteError::TerminalOutputError(e))?;

        self.pointer_x += 1;

        Ok(())
    }
}

impl<'a, T: TerminalWriter> TextArea<T> for AreaWriter<'a, T> {
    fn set_color(&mut self, color: TerminalColor) {
        self.color = color;
    }

    fn write_string(&mut self, string: &str) -> Result<(), TerminalWriteError<T::Error>> {
        for c in string.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, character: char) -> Result<(), TerminalWriteError<T::Error>> {
        let color = self.color;
        self.write(character, color)
    }

    fn write(&mut self, character: char, color: TerminalColor) -> Result<(), TerminalWriteError<T::Error>> {
        match character {
            '\n' => self.new_line()?,
            character => self.write_single_char(character, color)?,
        }

        Ok(())
    }

    fn backspace(&mut self) -> Result<(), TerminalWriteError<T::Error>> {
        // Check if any characters can be backspaced
        if self.pointer_y > self.origin_y || self.pointer_x > self.origin_x {
            // If current line empty, move up to above line
            if self.pointer_x == self.origin_x {
                self.pointer_y -= 1;
                self.pointer_x = self.origin_x + self.width;
            }

            self.pointer_x -= 1;

            let color = self.color;
            self.writer.lock().set(self.pointer_x, self.pointer_y, b' ', color)
                .map_err(|e| TerminalWriteError::TerminalOutputError(e))?;
        }

        Ok(())
    }

    fn new_line(&mut self) -> Result<(), TerminalWriteError<T::Error>> {
        if self.pointer_y < self.origin_y + self.height {
            self.pointer_y += 1;
            self.pointer_x = self.origin_x;
        }
        Ok(())
    }
}

impl<'a, T: TerminalWriter> fmt::Write for AreaWriter<'a, T> {
    fn write_str(&mut self, str: &str) -> Result<(), fmt::Error> {
        self.write_string(str).map_err(|_| fmt::Error)
    }
}

pub fn stdout_print(args: fmt::Arguments) {
    use core::fmt::Write;
    STDOUT.lock().write_fmt(args).unwrap();
}

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::drivers::terminal::text_area::stdout_print(format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
