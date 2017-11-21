use drivers::vga::{VgaWriter, Color};
use spin::Mutex;

pub static WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::new());

#[derive(Debug)]
pub enum TerminalWriteError<T> {
    TerminalOutputError(T)
}

/// Interface providing output to a terminal
pub trait TerminalWriter {
    type Error;

    /// Sets a character at the given cell
    fn set(&mut self, column: usize, row: usize, character: u8, color: TerminalColor) -> Result<(), Self::Error>;
}

/// Represents a foreground and background color for a character
#[derive(Copy, Clone)]
pub struct TerminalColor {
    pub foreground: Color,
    pub background: Color,
}

impl TerminalColor {
    pub const fn new(foreground: Color, background: Color) -> Self {
        TerminalColor {
            foreground: foreground,
            background: background,
        }
    }
}
