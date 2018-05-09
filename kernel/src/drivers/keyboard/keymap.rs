///
///
/// This module contains a list of US QWERTY key code constants, based around rows/columns on a
/// keyboard. This is used because, for example, in a game using WASD, you're looking for the
/// characters in that position, not those characters specifically.
///
/// All non-character codes can represent the same key on any keyboard layout.
#[allow(dead_code)] // Dead keys for completeness
pub mod codes {

    pub const ESCAPE: u8 = code(0, 0);
    pub const F1: u8 = code(1, 0);
    pub const F2: u8 = code(2, 0);
    pub const F3: u8 = code(3, 0);
    pub const F4: u8 = code(4, 0);
    pub const F5: u8 = code(5, 0);
    pub const F6: u8 = code(6, 0);
    pub const F7: u8 = code(7, 0);
    pub const F8: u8 = code(8, 0);
    pub const F9: u8 = code(9, 0);
    pub const F10: u8 = code(10, 0);
    pub const F11: u8 = code(11, 0);
    pub const F12: u8 = code(12, 0);
    pub const PRINT_SCREEN: u8 = code(13, 0);
    pub const SCROLL_LOCK: u8 = code(14, 0);
    pub const PAUSE: u8 = code(15, 0);
    pub const BACK_TICK: u8 = code(0, 1);
    pub const KEY_1: u8 = code(1, 1);
    pub const KEY_2: u8 = code(2, 1);
    pub const KEY_3: u8 = code(3, 1);
    pub const KEY_4: u8 = code(4, 1);
    pub const KEY_5: u8 = code(5, 1);
    pub const KEY_6: u8 = code(6, 1);
    pub const KEY_7: u8 = code(7, 1);
    pub const KEY_8: u8 = code(8, 1);
    pub const KEY_9: u8 = code(9, 1);
    pub const KEY_0: u8 = code(10, 1);
    pub const MINUS: u8 = code(11, 1);
    pub const EQUALS: u8 = code(12, 1);
    pub const BACKSPACE: u8 = code(13, 1);
    pub const INSERT: u8 = code(14, 1);
    pub const HOME: u8 = code(15, 1);
    pub const PAGE_UP: u8 = code(16, 1);
    pub const NUM_LOCK: u8 = code(17, 1);
    pub const NUM_PAD_FORWARD_SLASH: u8 = code(18, 1);
    pub const NUM_PAD_ASTERISK: u8 = code(19, 1);
    pub const NUM_PAD_MINUS: u8 = code(20, 1);
    pub const TAB: u8 = code(0, 2);
    pub const Q: u8 = code(1, 2);
    pub const W: u8 = code(2, 2);
    pub const E: u8 = code(3, 2);
    pub const R: u8 = code(4, 2);
    pub const T: u8 = code(5, 2);
    pub const Y: u8 = code(6, 2);
    pub const U: u8 = code(7, 2);
    pub const I: u8 = code(8, 2);
    pub const O: u8 = code(9, 2);
    pub const P: u8 = code(10, 2);
    pub const SQUARE_BRACKET_OPEN: u8 = code(11, 2);
    pub const SQUARE_BRACKET_CLOSE: u8 = code(12, 2);
    pub const BACK_SLASH: u8 = code(13, 2);
    pub const DELETE: u8 = code(14, 2);
    pub const END: u8 = code(15, 2);
    pub const PAGE_DOWN: u8 = code(16, 2);
    pub const NUM_PAD_7: u8 = code(17, 2);
    pub const NUM_PAD_8: u8 = code(18, 2);
    pub const NUM_PAD_9: u8 = code(19, 2);
    pub const CAPS_LOCK: u8 = code(0, 3);
    pub const A: u8 = code(1, 3);
    pub const S: u8 = code(2, 3);
    pub const D: u8 = code(3, 3);
    pub const F: u8 = code(4, 3);
    pub const G: u8 = code(5, 3);
    pub const H: u8 = code(6, 3);
    pub const J: u8 = code(7, 3);
    pub const K: u8 = code(8, 3);
    pub const L: u8 = code(9, 3);
    pub const SEMI_COLON: u8 = code(10, 3);
    pub const SINGLE_QUOTE: u8 = code(11, 3);
    pub const ENTER: u8 = code(12, 3);
    pub const NUM_PAD_4: u8 = code(13, 3);
    pub const NUM_PAD_5: u8 = code(14, 3);
    pub const NUM_PAD_6: u8 = code(15, 3);
    pub const NUM_PAD_PLUS: u8 = code(16, 3);
    pub const LEFT_SHIFT: u8 = code(0, 4);
    pub const Z: u8 = code(1, 4);
    pub const X: u8 = code(2, 4);
    pub const C: u8 = code(3, 4);
    pub const V: u8 = code(4, 4);
    pub const B: u8 = code(5, 4);
    pub const N: u8 = code(6, 4);
    pub const M: u8 = code(7, 4);
    pub const COMMA: u8 = code(8, 4);
    pub const PERIOD: u8 = code(9, 4);
    pub const FORWARD_SLASH: u8 = code(10, 4);
    pub const RIGHT_SHIFT: u8 = code(11, 4);
    pub const UP_ARROW: u8 = code(12, 4);
    pub const NUM_PAD_1: u8 = code(13, 4);
    pub const NUM_PAD_2: u8 = code(14, 4);
    pub const NUM_PAD_3: u8 = code(15, 4);
    pub const LEFT_CONTROL: u8 = code(0, 5);
    pub const LEFT_WIN: u8 = code(1, 5);
    pub const LEFT_ALT: u8 = code(2, 5);
    pub const SPACE: u8 = code(3, 5);
    pub const RIGHT_ALT: u8 = code(4, 5);
    pub const RIGHT_WIN: u8 = code(5, 5);
    pub const FUNCTION: u8 = code(6, 5);
    pub const RIGHT_CONTROL: u8 = code(7, 5);
    pub const LEFT_ARROW: u8 = code(8, 5);
    pub const DOWN_ARROW: u8 = code(9, 5);
    pub const RIGHT_ARROW: u8 = code(10, 5);
    pub const NUM_PAD_0: u8 = code(11, 5);
    pub const NUM_PAD_DELETE: u8 = code(12, 5);
    pub const NUM_PAD_ENTER: u8 = code(13, 5);

    /// Gets the Flower keycode for a key based on its row and column.
    const fn code(column: u8, row: u8) -> u8 {
        (column & 0x1F) | (row & 0x7) << 5
    }
}

/// Gets the US QWERTY character(s) for the given Flower keycode. The first element represents the
/// lower-case, and the second the upper.
pub fn get_us_qwerty_char(keycode: u8) -> Option<(char, char)> {
    match keycode {
        codes::KEY_1 => Some(('1', '!')),
        codes::KEY_2 => Some(('2', '@')),
        codes::KEY_3 => Some(('3', '#')),
        codes::KEY_4 => Some(('4', '$')),
        codes::KEY_5 => Some(('5', '%')),
        codes::KEY_6 => Some(('6', '^')),
        codes::KEY_7 => Some(('7', '&')),
        codes::KEY_8 => Some(('8', '*')),
        codes::KEY_9 => Some(('9', '(')),
        codes::KEY_0 => Some(('0', ')')),
        codes::MINUS => Some(('-', '_')),
        codes::EQUALS => Some(('=', '+')),
        codes::BACKSPACE => Some(('\x08', '\x08')),
        codes::TAB => Some(('\t', '\t')),
        codes::Q => Some(('q', 'Q')),
        codes::W => Some(('w', 'W')),
        codes::E => Some(('e', 'E')),
        codes::R => Some(('r', 'R')),
        codes::T => Some(('t', 'T')),
        codes::Y => Some(('y', 'Y')),
        codes::U => Some(('u', 'U')),
        codes::I => Some(('i', 'I')),
        codes::O => Some(('o', 'O')),
        codes::P => Some(('p', 'P')),
        codes::SQUARE_BRACKET_OPEN => Some(('[', '{')),
        codes::SQUARE_BRACKET_CLOSE => Some((']', '}')),
        codes::ENTER => Some(('\n', '\n')),
        codes::A => Some(('a', 'A')),
        codes::S => Some(('s', 'S')),
        codes::D => Some(('d', 'D')),
        codes::F => Some(('f', 'F')),
        codes::G => Some(('g', 'G')),
        codes::H => Some(('h', 'H')),
        codes::J => Some(('j', 'J')),
        codes::K => Some(('k', 'K')),
        codes::L => Some(('l', 'L')),
        codes::SEMI_COLON => Some((';', ':')),
        codes::SINGLE_QUOTE => Some(('\'', '\"')),
        codes::BACK_TICK => Some(('`', '~')),
        codes::BACK_SLASH => Some(('\\', '|')),
        codes::Z => Some(('z', 'Z')),
        codes::X => Some(('x', 'X')),
        codes::C => Some(('c', 'C')),
        codes::V => Some(('v', 'V')),
        codes::B => Some(('b', 'B')),
        codes::N => Some(('n', 'N')),
        codes::M => Some(('m', 'M')),
        codes::COMMA => Some((',', '<')),
        codes::PERIOD => Some(('.', '>')),
        codes::FORWARD_SLASH => Some(('/', '?')),
        codes::SPACE => Some((' ', ' ')),
        _ => None,
    }
}

/// Gets the Flower keycode for the given PS/2 scanset 2 scancode
pub fn get_code_ps2_set_2(scancode: u8) -> Option<u8> {
    match scancode {
        0x01 => Some(codes::F9),
        0x03 => Some(codes::F5),
        0x04 => Some(codes::F3),
        0x05 => Some(codes::F1),
        0x06 => Some(codes::F2),
        0x07 => Some(codes::F12),
        0x09 => Some(codes::F10),
        0x0A => Some(codes::F8),
        0x0B => Some(codes::F6),
        0x0C => Some(codes::F4),
        0x0D => Some(codes::TAB),
        0x0E => Some(codes::BACK_TICK),
        0x11 => Some(codes::LEFT_ALT),
        0x12 => Some(codes::LEFT_SHIFT),
        0x14 => Some(codes::LEFT_CONTROL),
        0x15 => Some(codes::Q),
        0x16 => Some(codes::KEY_1),
        0x1A => Some(codes::Z),
        0x1B => Some(codes::S),
        0x1C => Some(codes::A),
        0x1D => Some(codes::W),
        0x1E => Some(codes::KEY_2),
        0x21 => Some(codes::C),
        0x22 => Some(codes::X),
        0x23 => Some(codes::D),
        0x24 => Some(codes::E),
        0x25 => Some(codes::KEY_4),
        0x26 => Some(codes::KEY_3),
        0x29 => Some(codes::SPACE),
        0x2A => Some(codes::V),
        0x2B => Some(codes::F),
        0x2C => Some(codes::T),
        0x2D => Some(codes::R),
        0x2E => Some(codes::KEY_5),
        0x31 => Some(codes::N),
        0x32 => Some(codes::B),
        0x33 => Some(codes::H),
        0x34 => Some(codes::G),
        0x35 => Some(codes::Y),
        0x36 => Some(codes::KEY_6),
        0x3A => Some(codes::M),
        0x3B => Some(codes::J),
        0x3C => Some(codes::U),
        0x3D => Some(codes::KEY_7),
        0x3E => Some(codes::KEY_8),
        0x41 => Some(codes::COMMA),
        0x42 => Some(codes::K),
        0x43 => Some(codes::I),
        0x44 => Some(codes::O),
        0x45 => Some(codes::KEY_0),
        0x46 => Some(codes::KEY_9),
        0x49 => Some(codes::PERIOD),
        0x4A => Some(codes::FORWARD_SLASH),
        0x4B => Some(codes::L),
        0x4C => Some(codes::SEMI_COLON),
        0x4D => Some(codes::P),
        0x4E => Some(codes::MINUS),
        0x52 => Some(codes::SINGLE_QUOTE),
        0x54 => Some(codes::SQUARE_BRACKET_OPEN),
        0x55 => Some(codes::EQUALS),
        0x58 => Some(codes::CAPS_LOCK),
        0x59 => Some(codes::RIGHT_SHIFT),
        0x5A => Some(codes::ENTER),
        0x5B => Some(codes::SQUARE_BRACKET_CLOSE),
        0x5D => Some(codes::BACK_SLASH),
        0x66 => Some(codes::BACKSPACE),
        0x69 => Some(codes::NUM_PAD_1),
        0x6B => Some(codes::NUM_PAD_4),
        0x6C => Some(codes::NUM_PAD_7),
        0x70 => Some(codes::NUM_PAD_0),
        0x71 => Some(codes::NUM_PAD_DELETE),
        0x72 => Some(codes::NUM_PAD_2),
        0x73 => Some(codes::NUM_PAD_5),
        0x74 => Some(codes::NUM_PAD_6),
        0x75 => Some(codes::NUM_PAD_8),
        0x76 => Some(codes::ESCAPE),
        0x77 => Some(codes::NUM_LOCK),
        0x78 => Some(codes::F11),
        0x79 => Some(codes::NUM_PAD_PLUS),
        0x7A => Some(codes::NUM_PAD_3),
        0x7B => Some(codes::NUM_PAD_MINUS),
        0x7C => Some(codes::NUM_PAD_ASTERISK),
        0x7D => Some(codes::NUM_PAD_9),
        0x7E => Some(codes::SCROLL_LOCK),
        0x83 => Some(codes::F7),
        _ => None,
    }
}

/// Gets the Flower keycode for the given PS/2 extended scanset 2 scancode
pub fn get_extended_code_ps2_set_2(extended_code: u8) -> Option<u8> {
    match extended_code {
        0x11 => Some(codes::RIGHT_ALT),
        0x14 => Some(codes::RIGHT_CONTROL),
        0x4A => Some(codes::NUM_PAD_FORWARD_SLASH),
        0x5A => Some(codes::NUM_PAD_ENTER),
        0x69 => Some(codes::END),
        0x6C => Some(codes::HOME),
        0x70 => Some(codes::INSERT),
        0x71 => Some(codes::DELETE),
        0x7A => Some(codes::PAGE_DOWN),
        0x7D => Some(codes::PAGE_UP),
        _ => None,
    }
}
