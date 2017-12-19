from_discriminator! {
    /// Represents generic flower colors, based off of VGA's color set
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
}

#[derive(Debug, Copy, Clone)]
pub struct ColorPair {
    pub foreground: Color,
    pub background: Color,
}

impl ColorPair {
    #[allow(dead_code)] // Completeness
    pub const fn new(foreground: Color, background: Color) -> Self {
        ColorPair { foreground, background }
    }
}

impl Default for ColorPair {
    fn default() -> Self {
        ColorPair {
            foreground: Color::White,
            background: Color::Black
        }
    }
}

macro_rules! color {
    ($foreground:ident, $background:ident) => {
        ::color::ColorPair {
            foreground: ::color::Color::$foreground,
            background: ::color::Color::$background,
        }
    };

    ($foreground:ident on $background:ident) => {
        color!($foreground, $background)
    };
}