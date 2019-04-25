//! # Terminal Driver
//!
//! The terminal driver's goal is to provide text writer access to outputs such as VGA.
//! The driver accomplishes this through two trait categories
//!  - [TerminalWriter]- The terminal writer handles raw access to an output, for instance raw
//!    access to the vga buffer
//!  - [Terminal] - The terminal is a high-level wrapper of the [TerminalWriter],
//!    handling the cursor or tasks such as line-wrapping
//!
//!
//! The terminal driver also has an `STDOUT`, which is the standard output for terminals,
//! generally writing to VGA. This can be invoked through the `print!` and `println!` macros,
//! or directly referencing it through `drivers::terminal::STDOUT`

use crate::color::{Color, ColorPair};
use core::fmt::{self, Debug, Write};
use core::ops::Add;
use core::result::Result;
use crate::drivers::vga;
use spin::RwLock;

#[cfg(not(test))]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::terminal::stdout_print(format_args!($($arg)*));
    });
}

#[cfg(not(test))]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Writes formatted string to stdout, for print macro use
#[cfg(not(test))]
pub fn stdout_print(args: fmt::Arguments) {
    STDOUT.write().write_fmt(args).unwrap();
}

/// A standard output terminal
pub static STDOUT: RwLock<Stdout> = RwLock::new(Stdout(&vga::WRITER));

/// The standard output. You should not assume that the `Other` variant will
/// always carry a `()`.
// Crate public writer for `panic_fmt` to construct
pub struct Stdout<'a>(pub(crate) &'a RwLock<vga::VgaWriter>);

impl<'a> TerminalOutput<()> for Stdout<'a> {
    fn color_supported(&self, color: Color) -> bool {
        self.0.read().color_supported(color)
    }

    fn resolution(&self) -> Result<Resolution, TerminalOutputError<()>> {
        self.0.read().resolution()
    }

    fn cursor_pos(&self) -> Result<Point, TerminalOutputError<()>> {
        self.0.read().cursor_pos()
    }

    fn set_cursor_pos(&mut self, point: Point) -> Result<(), TerminalOutputError<()>> {
        self.0.write().set_cursor_pos(point)
    }

    fn color(&self) -> Result<ColorPair, TerminalOutputError<()>> {
        self.0.read().color()
    }

    fn set_color(&mut self, color: ColorPair) -> Result<(), TerminalOutputError<()>> {
        self.0.write().set_color(color)
    }

    fn set_char(&mut self, char: TerminalCharacter, point: Point) -> Result<(), TerminalOutputError<()>> {
        self.0.write().set_char(char, point)
    }

    fn write_colored(&mut self, character: char, color: ColorPair) -> Result<(), TerminalOutputError<()>> {
        self.0.write().write_colored(character, color)
    }

    fn clear_line(&mut self, y: usize) -> Result<(), TerminalOutputError<()>> {
        self.0.write().clear_line(y)
    }

    fn clear(&mut self) -> Result<(), TerminalOutputError<()>> {
        self.0.write().clear()
    }

    fn scroll_down(&mut self, lines: usize) -> Result<(), TerminalOutputError<()>> {
        self.0.write().scroll_down(lines)
    }
}

impl<'a> Write for Stdout<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write_string(s).map_err(|_| fmt::Error)
    }
}

/// A general [TerminalOutput] error
#[derive(Debug)]
#[allow(dead_code)] // Dead variants for completeness
pub enum TerminalOutputError<E: Debug> {
    /// Backspacing is not supported by the terminal
    BackspaceUnsupported,
    /// Backspacing is unavailable
    BackspaceUnavailable(BackspaceUnavailableCause),
    /// Setting characters at position is unsupported
    SetCharacterUnsupported,
    /// Cursor is unsupported
    CursorUnsupported,
    /// The given point was attempted to be accessed but is out of bounds
    OutOfBounds(Point),
    /// The given color is not supported by the terminal
    SpecificColorUnsupported(Color),
    /// The terminal does not support any color, i.e does not support coloring at all (e.g serial)
    ColorUnsupported,
    /// Clearing the whole/a line in the terminal is not supported
    ClearUnsupported,
    /// Terminal is a stream (and has no resolution)
    StreamWithoutResolution,
    /// An error with no other representation
    Other(E),
}

#[derive(Debug)]
#[allow(dead_code)] // Dead variants for completeness
pub enum BackspaceUnavailableCause {
    Disabled,
    TopOfTerminal,
}

/// Represents a character in a terminal, containing a character and color
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TerminalCharacter {
    pub character: char,
    pub color: ColorPair,
}

impl TerminalCharacter {
    pub const fn new(character: char, color: ColorPair) -> Self {
        TerminalCharacter { character, color }
    }
}

/// Represents a 2d point in a terminal. Origin is __bottom left__
#[derive(Default, Debug, Copy, Clone, Eq, PartialOrd, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    /// Creates a new point at the given coordinate
    pub const fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

/// Represents a terminal's resolution.
///
/// # Note
/// although the fields are the same as [Point], they *are* semantically different
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

impl Resolution {
    pub const fn new(x: usize, y: usize) -> Self {
        Resolution { x, y }
    }
    pub fn center(&self) -> Point {
        Point {
            x: (self.x - 1) / 2,
            y: (self.y - 1) / 2,
        }
    }
}

/// A writable terminal
///
/// # Note
///
/// The default implementations may not be the most efficient possible for your usecase - if you can
/// implement it more efficiently, then do.
pub trait TerminalOutput<E: Debug> {
    /// Check if a color is supported by this terminal
    fn color_supported(&self, color: Color) -> bool;

    /// The resolution of the [TerminalWriter].
    fn resolution(&self) -> Result<Resolution, TerminalOutputError<E>>;

    /// Checks if a point is in bounds of the text area
    fn in_bounds(&self, p: Point) -> Result<bool, TerminalOutputError<E>> {
        Ok(p.x < self.resolution()?.x && p.y < self.resolution()?.y)
    }

    /// Gets position of this terminal's cursor
    fn cursor_pos(&self) -> Result<Point, TerminalOutputError<E>>;

    /// Sets the position of this terminal's cursor
    ///
    /// # Implementation Note
    ///
    /// This should check whether the point is in bounds
    fn set_cursor_pos(&mut self, point: Point) -> Result<(), TerminalOutputError<E>>;

    /// Gets the color this terminal is using
    fn color(&self) -> Result<ColorPair, TerminalOutputError<E>>;

    /// Sets the color for this terminal to use
    ///
    /// # Implementation Note
    ///
    /// This should check whether the color is supported by this terminal
    fn set_color(&mut self, color: ColorPair) -> Result<(), TerminalOutputError<E>>;

    /// Sets the character at a position
    ///
    /// # Implementation Note
    ///
    /// This should check whether the point is in bounds
    fn set_char(&mut self, char: TerminalCharacter, point: Point) -> Result<(), TerminalOutputError<E>>;

    /// Writes a colored character to this terminal
    fn write_colored(&mut self, character: char, color: ColorPair) -> Result<(), TerminalOutputError<E>>;

    /// Writes a character to this terminal with the current set color
    fn write(&mut self, character: char) -> Result<(), TerminalOutputError<E>> {
        self.write_colored(character, self.color()?)
    }

    /// Writes a string to this terminal with the current set color
    fn write_string(&mut self, str: &str) -> Result<(), TerminalOutputError<E>> {
        self.write_string_colored(str, self.color()?)
    }

    /// Writes a colored string to this terminal
    fn write_string_colored(&mut self, str: &str, color: ColorPair) -> Result<(), TerminalOutputError<E>> {
        for character in str.chars() {
            self.write_colored(character, color)?;
        }

        Ok(())
    }

    /// Clears one line with the current background color
    ///
    /// # Note
    ///
    /// This is not defaultedly implemented
    fn clear_line(&mut self, y: usize) -> Result<(), TerminalOutputError<E>>;

    /// Clears the screen with the current background color
    fn clear(&mut self) -> Result<(), TerminalOutputError<E>>;

    /// Scrolls the terminal down
    fn scroll_down(&mut self, lines: usize) -> Result<(), TerminalOutputError<E>>;

    /// Writes a newline to this terminal, resetting cursor position
    fn new_line(&mut self) -> Result<(), TerminalOutputError<E>> {
        let mut pos = self.cursor_pos()?;
        pos.x = 0;

        if pos.y > 0 {
            pos.y -= 1;
        } else {
            self.scroll_down(1)?;
        }

        self.set_cursor_pos(pos)
    }

    /// Backspaces one character
    fn backspace(&mut self) -> Result<(), TerminalOutputError<E>> {
        if self.cursor_pos()? == Point::new(0, 0) {
            return Err(TerminalOutputError::BackspaceUnavailable(
                BackspaceUnavailableCause::TopOfTerminal)
            );
        }

        if self.cursor_pos()?.x == 0 {
            if self.cursor_pos()?.y != self.resolution()?.y {
                self.set_cursor_pos(Point {
                    x: self.resolution()?.x - 1,
                    y: self.cursor_pos()?.y + 1,
                })?;
            }
        } else {
            self.set_cursor_pos(Point {
                x: self.cursor_pos()?.x - 1,
                ..self.cursor_pos()?
            })?;
        }

        let blank = TerminalCharacter::new(' ', ColorPair::new(
            self.color()?.background, self.color()?.background,
        ));

        self.set_char(blank, self.cursor_pos()?)
    }
}
