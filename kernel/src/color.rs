use core::convert::TryFrom;

/// Represents generic flower colors
#[allow(dead_code)] // dead variants for completeness
#[derive(Debug, Copy, Clone)]
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
        match value {
            0 => Ok(Color::Black),
            1 => Ok(Color::Blue),
            2 => Ok(Color::Green),
            3 => Ok(Color::Cyan),
            4 => Ok(Color::Red),
            5 => Ok(Color::Magenta),
            6 => Ok(Color::Brown),
            7 => Ok(Color::LightGray),
            8 => Ok(Color::DarkGray),
            9 => Ok(Color::LightBlue),
            10 => Ok(Color::LightGreen),
            11 => Ok(Color::LightCyan),
            12 => Ok(Color::LightRed),
            13 => Ok(Color::Pink),
            14 => Ok(Color::Yellow),
            15 => Ok(Color::White),
            code => Err(ColorCodeOutOfBounds(code))
        }
    }
}
