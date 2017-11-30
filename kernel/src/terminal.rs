//! # Terminal Driver
//!
//! The terminal driver's goal is to provide simple text writer access to outputs such as VGA.
//! The driver accomplishes this through two traits:
//!  - [TerminalWriter] : The terminal writer handles raw access to an output, such as raw access to write to a point in the VGA buffer
//!  - [Terminal] : The terminal is a high-level wrapper of the [TerminalWriter], handling the cursor or tasks such as line-wrapping
//!
//! The terminal driver also has an `STDOUT`, which is the standard output for terminals, generally writing to VGA.
//! This can be invoked through the `print!` and `println!` macros, or directly referencing it through `drivers::terminal::STDOUT`

use core::ops::Add;
use core::fmt;
use core::result::Result;
use spin::Mutex;
use drivers::vga::{self, VgaWriter};
use color::Color;

pub static WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new());

/// A standard output terminal, generally writing to VGA
pub static STDOUT: Mutex<TextArea<VgaWriter>> = Mutex::new(TextArea::new(&WRITER, Point::new(0, 0), Point::new(vga::RESOLUTION_X, vga::RESOLUTION_Y)));

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::terminal::stdout_print(format_args!($($arg)*));
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Writes formatted string to stdout, for print macro use
pub fn stdout_print(args: fmt::Arguments) {
    use core::fmt::Write;
    STDOUT.lock().write_fmt(args).unwrap();
}

/// A general error terminal error
#[derive(Debug)]
pub enum TerminalWriteError<T: TerminalWriter> {
    OutputError(T::Error),
    BackspaceUnsupported,
    BackspaceUnavailable,
    OutOfBounds(Point),
}

/// Represents a character in a terminal, containing a character and color.
#[derive(Debug, Copy, Clone)]
pub struct TerminalCharacter {
    pub character: char,
    pub color: TerminalColor,
}

/// Represents a full color in the terminal with a background and foreground color.
#[derive(Debug, Copy, Clone)]
pub struct TerminalColor {
    pub foreground: Color,
    pub background: Color,
}

impl TerminalColor {
    /// Creates a new terminal color with a foreground and background
    pub const fn new(foreground: Color, background: Color) -> Self {
        TerminalColor {
            foreground: foreground,
            background: background,
        }
    }
}

/// Represents a 2d point in a terminal.
#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    /// Creates a new point at the given coordinate
    pub const fn new(x: usize, y: usize) -> Self {
        Point {
            x: x,
            y: y,
        }
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// A writable unsized terminal. Works as a stream without character positions.
pub trait Terminal<'a, T: TerminalWriter + 'a> {
    /// Sets the color for this terminal to use
    fn set_color(&mut self, color: TerminalColor) -> Result<(), TerminalWriteError<T>>;

    /// Writes a character to this terminal with the current set color
    fn write(&mut self, char: char) -> Result<(), TerminalWriteError<T>>;

    /// Writes a string to this terminal with the current set color
    fn write_string(&mut self, str: &str) -> Result<(), TerminalWriteError<T>>;

    /// Writes a colored character to this terminal
    fn write_colored(&mut self, char: char, color: TerminalColor) -> Result<(), TerminalWriteError<T>>;

    /// Writes a colored string to this terminal
    fn write_string_colored(&mut self, str: &str, color: TerminalColor) -> Result<(), TerminalWriteError<T>>;

    /// Backspaces the previously written character in this terminal
    fn backspace(&mut self) -> Result<(), TerminalWriteError<T>>;
}

/// A writable terminal with bounds and a 2d cursor that can be moved.
pub trait SizedTerminal<'a, T: TerminalWriter + 'a>: Terminal<'a, T> {
    /// Sets the position of this terminal's cursor
    fn set_cursor(&mut self, point: Point) -> Result<(), TerminalWriteError<T>>;

    /// Moves the cursor down on y and returns to origin on x
    fn new_line(&mut self) -> Result<(), TerminalWriteError<T>>;

    /// Returns true if the given point is inside this terminal
    fn is_valid(&self, point: Point) -> bool;
}

/// A raw interface that unsized terminals can write to.
pub trait TerminalWriter {
    type Error;

    /// Writes a character through this writer
    fn write(&mut self, char: TerminalCharacter) -> Result<(), Self::Error>;
}

/// A raw interface that sized terminals can write to.
pub trait SizedTerminalWriter: TerminalWriter {
    /// Sets a character at the given position through this writer
    fn set(&mut self, point: Point, char: TerminalCharacter) -> Result<(), Self::Error>;
}

/// A terminal with a constant stream of characters.
#[allow(dead_code)] // to be used as API
pub struct StreamTerminal<'a, T: TerminalWriter + 'a> {
    writer: &'a Mutex<T>,
    color: TerminalColor,
}

#[allow(dead_code)] // to be used as API
impl<'a, T: TerminalWriter + 'a> StreamTerminal<'a, T> {
    /// Creates a new terminal with a writer Mutex for the terminal to write to
    pub const fn new(writer: &'a Mutex<T>) -> Self {
        StreamTerminal {
            writer: writer,
            color: TerminalColor::new(Color::White, Color::Black),
        }
    }
}

impl<'a, T: TerminalWriter + 'a> Terminal<'a, T> for StreamTerminal<'a, T> {
    fn set_color(&mut self, color: TerminalColor) -> Result<(), TerminalWriteError<T>> {
        self.color = color;
        Ok(())
    }

    fn write(&mut self, char: char) -> Result<(), TerminalWriteError<T>> {
        let color = self.color;
        self.write_colored(char, color)
    }

    fn write_string(&mut self, str: &str) -> Result<(), TerminalWriteError<T>> {
        let color = self.color;
        self.write_string_colored(str, color)
    }

    fn write_colored(&mut self, char: char, color: TerminalColor) -> Result<(), TerminalWriteError<T>> {
        self.writer.lock().write(TerminalCharacter {
            character: char,
            color: color,
        }).map_err(|e| TerminalWriteError::OutputError(e))?;

        Ok(())
    }

    fn write_string_colored(&mut self, str: &str, color: TerminalColor) -> Result<(), TerminalWriteError<T>> {
        for char in str.chars() {
            self.write_colored(char, color)?;
        }
        Ok(())
    }

    fn backspace(&mut self) -> Result<(), TerminalWriteError<T>> {
        Err(TerminalWriteError::BackspaceUnsupported)
    }
}

/// A sized terminal text area.
pub struct TextArea<'a, T: SizedTerminalWriter + 'a> {
    writer: &'a Mutex<T>,
    min_point: Point,
    max_point: Point,
    cursor: Point,
    color: TerminalColor,
}

impl<'a, T: SizedTerminalWriter + 'a> TextArea<'a, T> {
    /// Creates a new TextArea with a writer Mutex for the terminal to write to and the given bounds
    pub const fn new(writer: &'a Mutex<T>, origin: Point, resolution: Point) -> Self {
        TextArea {
            writer: writer,
            min_point: origin,
            max_point: Point {
                x: origin.x + resolution.x,
                y: origin.y + resolution.y,
            },
            cursor: origin,
            color: TerminalColor::new(Color::White, Color::Black),
        }
    }
}

impl<'a, T: SizedTerminalWriter + 'a> Terminal<'a, T> for TextArea<'a, T> {
    fn set_color(&mut self, color: TerminalColor) -> Result<(), TerminalWriteError<T>> {
        self.color = color;
        Ok(())
    }

    fn write(&mut self, char: char) -> Result<(), TerminalWriteError<T>> {
        let color = self.color;
        self.write_colored(char, color)
    }

    fn write_string(&mut self, str: &str) -> Result<(), TerminalWriteError<T>> {
        let color = self.color;
        self.write_string_colored(str, color)
    }

    fn write_colored(&mut self, char: char, color: TerminalColor) -> Result<(), TerminalWriteError<T>> {
        let cursor = self.cursor;

        if self.is_valid(cursor) {
            match char {
                '\n' => self.new_line()?,
                c => {
                    self.writer.lock().set(cursor, TerminalCharacter {
                        character: c,
                        color: color,
                    }).map_err(|e| TerminalWriteError::OutputError(e))?;

                    self.cursor.x += 1;

                    // If cursor exceeded x limit, wrap to next line
                    if self.cursor.x > self.max_point.x {
                        self.new_line()?;
                    }
                }
            }

            Ok(())
        } else {
            Err(TerminalWriteError::OutOfBounds(cursor))
        }
    }

    fn write_string_colored(&mut self, str: &str, color: TerminalColor) -> Result<(), TerminalWriteError<T>> {
        for char in str.chars() {
            self.write_colored(char, color)?;
        }

        Ok(())
    }

    fn backspace(&mut self) -> Result<(), TerminalWriteError<T>> {
        // If backspace is possible
        if self.cursor.x > self.min_point.x || self.cursor.y > self.min_point.y {
            // If at start of line, move up a line, else move back in the current line
            if self.cursor.x == self.min_point.x {
                self.cursor.x = self.max_point.x;
                self.cursor.y -= 1;
            } else {
                self.cursor.x -= 1;
            }

            self.writer.lock().set(self.cursor, TerminalCharacter {
                character: ' ',
                color: self.color,
            }).map_err(|e| TerminalWriteError::OutputError(e))?;

            Ok(())
        } else {
            Err(TerminalWriteError::BackspaceUnavailable)
        }
    }
}

impl<'a, T: SizedTerminalWriter + 'a> SizedTerminal<'a, T> for TextArea<'a, T> {
    fn set_cursor(&mut self, point: Point) -> Result<(), TerminalWriteError<T>> {
        if self.is_valid(point) {
            self.cursor = point;
            Ok(())
        } else {
            Err(TerminalWriteError::OutOfBounds(point))
        }
    }

    fn new_line(&mut self) -> Result<(), TerminalWriteError<T>> {
        if self.cursor.y < self.max_point.y {
            self.cursor.x = self.min_point.x;
            self.cursor.y += 1;
            Ok(())
        } else {
            Err(TerminalWriteError::OutOfBounds(self.cursor))
        }
    }

    fn is_valid(&self, point: Point) -> bool {
        point.x >= self.min_point.x && point.y >= self.min_point.y && point.x < self.max_point.x && point.y < self.max_point.y
    }
}

/// Implemented to allow formatted strings to be written to text areas
impl<'a, T: SizedTerminalWriter + 'a> fmt::Write for TextArea<'a, T> {
    fn write_str(&mut self, str: &str) -> Result<(), fmt::Error> {
        self.write_string(str).map_err(|_| fmt::Error)?;
        Ok(())
    }
}
