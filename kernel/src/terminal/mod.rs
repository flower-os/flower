// TODO:
//
// TerminalInput
// Displaying cursor
// Disabling of backspace
// Disabling of edit to terminals

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

mod text_area;
pub use self::text_area::TextArea;

use core::ops::Add;
use core::fmt::{self, Debug, Write};
use core::marker::PhantomData;
use core::result::Result;
use spin::Mutex;
use drivers::vga;
use color::{Color, ColorPair};

/// A standard output terminal
pub static STDOUT: Mutex<Stdout> = Mutex::new(Stdout(&vga::WRITER));

/// The standard output. You should not assume that the `Other` variant will
/// always carry a `()`.
// Crate public writer for `panic_fmt` to use
pub struct Stdout<'a>(pub(crate) &'a Mutex<vga::VgaWriter>);

// TODO maybe rwlock for most things
impl<'a> TerminalOutput<()> for Stdout<'a> {
    fn color_supported(&self, color: Color) -> bool {
        self.0.lock().color_supported(color)
    }

    fn resolution(&self) -> Resolution {
        self.0.lock().resolution()
    }

    fn cursor_pos(&self) -> Point {
        self.0.lock().cursor_pos()
    }

    fn set_cursor_pos(&mut self, point: Point) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().set_cursor_pos(point)
    }

    fn color(&self) -> ColorPair {
        self.0.lock().color()
    }

    fn set_color(&mut self, color: ColorPair) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().set_color(color)
    }

    fn set_char(&mut self, char: TerminalCharacter, point: Point) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().set_char(char, point)
    }

    fn write_raw(&mut self, char: TerminalCharacter) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().write_raw(char)
    }

    fn clear_line(&mut self, y: usize) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().clear_line(y)
    }
    fn clear(&mut self) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().clear()
    }

    fn scroll_down(&mut self, lines: usize) -> Result<(), TerminalOutputError<()>> {
        self.0.lock().scroll_down(lines)
    }
}

impl<'a> Write for Stdout<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write_string(s).map_err(|_| fmt::Error)
    }
}

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

/// A general [TerminalOutput] error
#[derive(Debug)]
#[allow(dead_code)] // Dead variants for completeness
pub enum TerminalOutputError<E: Debug> {
    /// Backspacing is not supported by the terminal
    BackspaceUnsupported,
    /// Backspacing is unavailable
    BackspaceUnavailable(BackspaceUnavailableCause),
    /// The given point was attempted to be accessed but is out of bounds
    OutOfBounds(Point),
    Debug(Point, Point, Point),
    /// The given color is not supported by the terminal
    ColorUnsupported(Color),
    /// An error with no other representation
    Other(E),
}

#[derive(Debug)]
#[allow(dead_code)] // Dead variants for completeness
pub enum BackspaceUnavailableCause {
    Disabled,
    // TODO impl disabling backspace
    TopOfTerminal,
}

/// Represents a character in a terminal, containing a character and color
#[derive(Debug, Copy, Clone)]
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
#[derive(Default, Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
pub struct Resolution {
    pub x: usize,
    pub y: usize,
}

impl Resolution {
    pub const fn new(x: usize, y: usize) -> Self {
        Resolution { x, y }
    }
}

/// A writable terminal
// TODO: implement backspacing disabling
// TODO (not blocking merge) - resizing of terminal
pub trait TerminalOutput<E: Debug> {
    /// Check if a color is supported by this terminal
    fn color_supported(&self, color: Color) -> bool;

    /// The resolution of the [TerminalWriter].
    /// Although a Point may be within the bounds of the resolution, it may
    /// not be within the bounds of the terminal (e.g in a [TextArea]). The
    /// [in_bounds] method should be used to check whether the point is in bounds.
    fn resolution(&self) -> Resolution;

    /// Whether a point is in bounds of the terminal.
    ///
    /// ## Default behaviour
    ///
    /// By default, this method just checks whether the point is in bounds
    /// of the terminal's resolution. **This is not correct for things such as
    /// [TextArea]s.**
    fn in_bounds(&self, point: Point) -> bool {
        // TODO
        let x = self.resolution().x;
        let y = self.resolution().y;
        let p_x = point.x;
        let p_y = point.y;
        point.x < self.resolution().x && point.y < self.resolution().y
    }

    /// Gets position of this terminal's cursor
    fn cursor_pos(&self) -> Point;

    /// Sets the position of this terminal's cursor
    ///
    /// # Implementation Note
    ///
    /// This should check whether the point is in bounds
    fn set_cursor_pos(&mut self, point: Point) -> Result<(), TerminalOutputError<E>>;

    /// Gets the color this terminal is using
    fn color(&self) -> ColorPair;

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

    /// Writes a raw, n on-special[ TerminalCharacter] character to this terminal, wrapping if it
    /// has to
    ///
    /// # Implementation Note
    ///
    /// This should check if the color is supported by this terminal
    // TODO when to take character and when char and ColorPair?
    fn write_raw(&mut self, character: TerminalCharacter) -> Result<(), TerminalOutputError<E>>;

    /// Writes a colored, potentially-special character to this terminal
    fn write_colored(&mut self, character: char, color: ColorPair) -> Result<(), TerminalOutputError<E>> {
        match character {
            '\n' => self.new_line(),
            _ => {
                self.write_raw(TerminalCharacter { character, color })
            }
        }
    }

    /// Writes a character to this terminal with the current set color
    fn write(&mut self, character: char) -> Result<(), TerminalOutputError<E>> {
        let color = self.color();
        self.write_colored(character, color)
    }

    /// Writes a string to this terminal with the current set color
    fn write_string(&mut self, str: &str) -> Result<(), TerminalOutputError<E>> {
        let color = self.color();
        self.write_string_colored(str, color)
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
    // TODO scroll up with history?
    fn scroll_down(&mut self, lines: usize) -> Result<(), TerminalOutputError<E>>;

    /// Writes a newline to this terminal, resetting cursor position
    fn new_line(&mut self) -> Result<(), TerminalOutputError<E>> {
        let mut pos = self.cursor_pos();
        pos.x = 0;

        if pos.y > 0 {
            pos.y -= 1;
        } else {
            self.scroll_down(1)?; // TODO __new line needs to not be called from textarea__
        }

        self.set_cursor_pos(pos)
    }

    /// Backspaces one character
    fn backspace(&mut self) -> Result<(), TerminalOutputError<E>> {
        // TODO gegy, do pls
        unimplemented!()
    }
}

