use crate::drivers::keyboard::{Keycode, KeyCharMapping};

#[allow(dead_code)] // Dead keys for completeness
pub mod codes {
    //! # Codes
    //!
    //! This module contains a list of US QWERTY key code constants.

    use crate::drivers::keyboard::Keycode;

    pub const F1: Keycode = 0x00;
    pub const F2: Keycode = 0x01;
    pub const F3: Keycode = 0x02;
    pub const F4: Keycode = 0x03;
    pub const F5: Keycode = 0x04;
    pub const F6: Keycode = 0x05;
    pub const F7: Keycode = 0x06;
    pub const F8: Keycode = 0x07;
    pub const F9: Keycode = 0x08;
    pub const F10: Keycode = 0x09;
    pub const F11: Keycode = 0x0A;
    pub const F12: Keycode = 0x0B;

    pub const KEY_1: Keycode = 0x0C;
    pub const KEY_2: Keycode = 0x0D;
    pub const KEY_3: Keycode = 0x0E;
    pub const KEY_4: Keycode = 0x0F;
    pub const KEY_5: Keycode = 0x10;
    pub const KEY_6: Keycode = 0x11;
    pub const KEY_7: Keycode = 0x12;
    pub const KEY_8: Keycode = 0x13;
    pub const KEY_9: Keycode = 0x14;
    pub const KEY_0: Keycode = 0x15;

    pub const Q: Keycode = 0x16;
    pub const W: Keycode = 0x17;
    pub const E: Keycode = 0x18;
    pub const R: Keycode = 0x19;
    pub const T: Keycode = 0x1A;
    pub const Y: Keycode = 0x1B;
    pub const U: Keycode = 0x1C;
    pub const I: Keycode = 0x1D;
    pub const O: Keycode = 0x1E;
    pub const P: Keycode = 0x1F;
    pub const A: Keycode = 0x20;
    pub const S: Keycode = 0x21;
    pub const D: Keycode = 0x22;
    pub const F: Keycode = 0x23;
    pub const G: Keycode = 0x24;
    pub const H: Keycode = 0x25;
    pub const J: Keycode = 0x26;
    pub const K: Keycode = 0x27;
    pub const L: Keycode = 0x28;
    pub const Z: Keycode = 0x29;
    pub const X: Keycode = 0x2A;
    pub const C: Keycode = 0x2B;
    pub const V: Keycode = 0x2C;
    pub const B: Keycode = 0x2D;
    pub const N: Keycode = 0x2E;
    pub const M: Keycode = 0x2F;

    pub const SPACE: Keycode = 0x30;
    pub const EQUALS: Keycode = 0x31;
    pub const MINUS: Keycode = 0x32;
    pub const COMMA: Keycode = 0x33;
    pub const PERIOD: Keycode = 0x34;
    pub const SEMI_COLON: Keycode = 0x35;
    pub const SINGLE_QUOTE: Keycode = 0x36;
    pub const BACK_TICK: Keycode = 0x37;
    pub const SQUARE_BRACKET_OPEN: Keycode = 0x38;
    pub const SQUARE_BRACKET_CLOSE: Keycode = 0x39;
    pub const FORWARD_SLASH: Keycode = 0x3A;
    pub const BACK_SLASH: Keycode = 0x3B;
    pub const ESCAPE: Keycode = 0x3C;
    pub const ENTER: Keycode = 0x3D;
    pub const BACKSPACE: Keycode = 0x3E;
    pub const TAB: Keycode = 0x3F;

    pub const PRINT_SCREEN: Keycode = 0x40;
    pub const PAUSE: Keycode = 0x41;
    pub const INSERT: Keycode = 0x42;
    pub const DELETE: Keycode = 0x43;
    pub const HOME: Keycode = 0x44;
    pub const PAGE_UP: Keycode = 0x45;
    pub const PAGE_DOWN: Keycode = 0x46;
    pub const END: Keycode = 0x47;

    pub const FUNCTION: Keycode = 0x48;
    pub const LEFT_CONTROL: Keycode = 0x49;
    pub const RIGHT_CONTROL: Keycode = 0x4A;
    pub const LEFT_SHIFT: Keycode = 0x4B;
    pub const RIGHT_SHIFT: Keycode = 0x4C;
    pub const LEFT_WIN: Keycode = 0x4D;
    pub const RIGHT_WIN: Keycode = 0x4E;
    pub const LEFT_ALT: Keycode = 0x4F;
    pub const RIGHT_ALT: Keycode = 0x50;

    pub const SCROLL_LOCK: Keycode = 0x51;
    pub const NUM_LOCK: Keycode = 0x52;
    pub const CAPS_LOCK: Keycode = 0x53;
    pub const UP_ARROW: Keycode = 0x54;
    pub const LEFT_ARROW: Keycode = 0x55;
    pub const DOWN_ARROW: Keycode = 0x56;
    pub const RIGHT_ARROW: Keycode = 0x57;

    pub const NUM_PAD_0: Keycode = 0x58;
    pub const NUM_PAD_1: Keycode = 0x59;
    pub const NUM_PAD_2: Keycode = 0x5A;
    pub const NUM_PAD_3: Keycode = 0x5B;
    pub const NUM_PAD_4: Keycode = 0x5C;
    pub const NUM_PAD_5: Keycode = 0x5D;
    pub const NUM_PAD_6: Keycode = 0x5E;
    pub const NUM_PAD_7: Keycode = 0x5F;
    pub const NUM_PAD_8: Keycode = 0x60;
    pub const NUM_PAD_9: Keycode = 0x61;
    pub const NUM_PAD_PLUS: Keycode = 0x62;
    pub const NUM_PAD_MINUS: Keycode = 0x63;
    pub const NUM_PAD_ENTER: Keycode = 0x64;
    pub const NUM_PAD_PERIOD: Keycode = 0x65;
    pub const NUM_PAD_FORWARD_SLASH: Keycode = 0x66;
    pub const NUM_PAD_ASTERISK: Keycode = 0x67;
}

/// Gets the character(s) for the given Flower keycode.
pub fn get_us_qwerty_char(keycode: Keycode) -> KeyCharMapping {
    use self::codes::*;
    match keycode {
        KEY_1 => KeyCharMapping::Shifted('1', '!'),
        KEY_2 => KeyCharMapping::Shifted('2', '@'),
        KEY_3 => KeyCharMapping::Shifted('3', '#'),
        KEY_4 => KeyCharMapping::Shifted('4', '$'),
        KEY_5 => KeyCharMapping::Shifted('5', '%'),
        KEY_6 => KeyCharMapping::Shifted('6', '^'),
        KEY_7 => KeyCharMapping::Shifted('7', '&'),
        KEY_8 => KeyCharMapping::Shifted('8', '*'),
        KEY_9 => KeyCharMapping::Shifted('9', '('),
        KEY_0 => KeyCharMapping::Shifted('0', ')'),

        Q => KeyCharMapping::Capitalized('q', 'Q'),
        W => KeyCharMapping::Capitalized('w', 'W'),
        E => KeyCharMapping::Capitalized('e', 'E'),
        R => KeyCharMapping::Capitalized('r', 'R'),
        T => KeyCharMapping::Capitalized('t', 'T'),
        Y => KeyCharMapping::Capitalized('y', 'Y'),
        U => KeyCharMapping::Capitalized('u', 'U'),
        I => KeyCharMapping::Capitalized('i', 'I'),
        O => KeyCharMapping::Capitalized('o', 'O'),
        P => KeyCharMapping::Capitalized('p', 'P'),
        A => KeyCharMapping::Capitalized('a', 'A'),
        S => KeyCharMapping::Capitalized('s', 'S'),
        D => KeyCharMapping::Capitalized('d', 'D'),
        F => KeyCharMapping::Capitalized('f', 'F'),
        G => KeyCharMapping::Capitalized('g', 'G'),
        H => KeyCharMapping::Capitalized('h', 'H'),
        J => KeyCharMapping::Capitalized('j', 'J'),
        K => KeyCharMapping::Capitalized('k', 'K'),
        L => KeyCharMapping::Capitalized('l', 'L'),
        Z => KeyCharMapping::Capitalized('z', 'Z'),
        X => KeyCharMapping::Capitalized('x', 'X'),
        C => KeyCharMapping::Capitalized('c', 'C'),
        V => KeyCharMapping::Capitalized('v', 'V'),
        B => KeyCharMapping::Capitalized('b', 'B'),
        N => KeyCharMapping::Capitalized('n', 'N'),
        M => KeyCharMapping::Capitalized('m', 'M'),

        SPACE => KeyCharMapping::Single(' '),
        EQUALS => KeyCharMapping::Shifted('=', '+'),
        MINUS => KeyCharMapping::Shifted('-', '_'),
        COMMA => KeyCharMapping::Shifted(',', '<'),
        PERIOD => KeyCharMapping::Shifted('.', '>'),
        SEMI_COLON => KeyCharMapping::Shifted(';', ':'),
        SINGLE_QUOTE => KeyCharMapping::Shifted('\'', '\"'),
        BACK_TICK => KeyCharMapping::Shifted('`', '~'),
        SQUARE_BRACKET_OPEN => KeyCharMapping::Shifted('[', '{'),
        SQUARE_BRACKET_CLOSE => KeyCharMapping::Shifted(']', '}'),
        FORWARD_SLASH => KeyCharMapping::Shifted('/', '?'),
        BACK_SLASH => KeyCharMapping::Shifted('\\', '|'),
        ENTER => KeyCharMapping::Single('\n'),
        BACKSPACE => KeyCharMapping::Single('\x08'),
        TAB => KeyCharMapping::Single('\t'),

        NUM_PAD_0 => KeyCharMapping::NumLocked('0'),
        NUM_PAD_1 => KeyCharMapping::NumLocked('1'),
        NUM_PAD_2 => KeyCharMapping::NumLocked('2'),
        NUM_PAD_3 => KeyCharMapping::NumLocked('3'),
        NUM_PAD_4 => KeyCharMapping::NumLocked('4'),
        NUM_PAD_5 => KeyCharMapping::NumLocked('5'),
        NUM_PAD_6 => KeyCharMapping::NumLocked('6'),
        NUM_PAD_7 => KeyCharMapping::NumLocked('7'),
        NUM_PAD_8 => KeyCharMapping::NumLocked('8'),
        NUM_PAD_9 => KeyCharMapping::NumLocked('9'),

        NUM_PAD_PLUS => KeyCharMapping::Single('+'),
        NUM_PAD_MINUS => KeyCharMapping::Single('-'),
        NUM_PAD_ENTER => KeyCharMapping::Single('\n'),
        NUM_PAD_PERIOD => KeyCharMapping::NumLocked('.'),
        NUM_PAD_FORWARD_SLASH => KeyCharMapping::Single('/'),
        NUM_PAD_ASTERISK => KeyCharMapping::Single('*'),

        _ => KeyCharMapping::Empty,
    }
}
