use crate::drivers::ps2;
use crate::drivers::ps2::io::CommandIo;

use spin::Mutex;
use crate::drivers::keyboard::Keycode;

static INPUT_PARSER: Mutex<EventParser> = Mutex::new(EventParser::new());

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Scanset {
    One = 1,
    Two = 2,
    Three = 3,
}

bitflags! {
    pub struct LedFlags: u8 {
        /// If scroll lock is active
        const SCROLL_LOCK = 1 << 0;
        /// If number lock is active
        const NUMBER_LOCK = 1 << 1;
        /// If caps lock is active
        const CAPS_LOCK = 1 << 2;
    }
}

#[derive(Debug, Copy, Clone)]
struct Scancode {
    code: u8,
    extended: bool,
    make: bool,
}

struct EventParser {
    make_scancode: bool,
    extended_scancode: bool,
}

impl EventParser {
    const fn new() -> EventParser {
        EventParser {
            make_scancode: true,
            extended_scancode: false,
        }
    }

    pub fn next_event(&mut self) -> Option<Event> {
        while let Some(byte) = ps2::port::KEYBOARD_INPUT.next() {
            let event = self.parse_byte(byte);
            if event.is_some() {
                return event;
            }
        }
        None
    }

    fn parse_byte(&mut self, byte: u8) -> Option<Event> {
        match byte {
            0xAA => Some(Event::BatSuccess),
            0xFC => Some(Event::BatError),
            _ => {
                self.parse_scancode(byte).and_then(|scancode| {
                    match scanset_2::parse(scancode) {
                        Some(key) => Some(Event::Key { key, make: scancode.make }),
                        None => Some(Event::UnexpectedByte(scancode.code))
                    }
                })
            }
        }
    }

    fn parse_scancode(&mut self, byte: u8) -> Option<Scancode> {
        match byte {
            0xE0...0xE1 => self.extended_scancode = true,
            0xF0 => self.make_scancode = false,
            _ => {
                let scancode = Scancode {
                    code: byte,
                    make: self.make_scancode,
                    extended: self.extended_scancode,
                };
                self.make_scancode = true;
                self.extended_scancode = false;

                return Some(scancode);
            }
        }
        None
    }
}

pub struct Keyboard;

impl ps2::io::CommandIo for Keyboard {
    #[inline]
    fn send(command: u8) -> ps2::Result<()> {
        ps2::device::send_raw_device_command(command, false)
    }

    #[inline]
    fn send_data(command: u8, data: u8) -> ps2::Result<()> {
        ps2::device::send_raw_device_command_data(command, data, false)
    }

    fn read() -> ps2::Result<u8> {
        ps2::io::read(&ps2::io::DATA_PORT).ok_or(ps2::Error::ExpectedResponse)
    }
}

impl ps2::Device for Keyboard {
    #[inline]
    fn enable() -> ps2::Result<()> { ps2::Controller::enable_keyboard() }

    #[inline]
    fn disable() -> ps2::Result<()> { ps2::Controller::disable_keyboard() }

    #[inline]
    fn test() -> ps2::Result<bool> { ps2::Controller::test_keyboard() }
}

#[derive(Debug, Clone)]
pub enum Event {
    BatSuccess,
    BatError,
    Key { key: Keycode, make: bool },
    UnexpectedByte(u8),
}

impl Keyboard {
    #[inline]
    pub fn next_event() -> Option<Event> {
        INPUT_PARSER.lock().next_event()
    }

    #[inline]
    pub fn set_leds(flags: ps2::keyboard::LedFlags) -> ps2::Result<()> {
        Self::send_data(0xED, flags.bits())
    }

    #[inline]
    pub fn set_scanset(scanset: ps2::keyboard::Scanset) -> ps2::Result<()> {
        Self::send_data(0xF0, scanset as u8)
    }

    #[inline]
    pub fn get_scanset() -> ps2::Result<ps2::keyboard::Scanset> {
        Self::send_data(0xF0, 0)?;
        match Self::read()? {
            1 => Ok(ps2::keyboard::Scanset::One),
            2 => Ok(ps2::keyboard::Scanset::Two),
            3 => Ok(ps2::keyboard::Scanset::Three),
            v => Err(ps2::Error::UnexpectedResponse(v)),
        }
    }
}

mod scanset_2 {
    use crate::drivers::keyboard::keymap::codes::*;
    use crate::drivers::keyboard::Keycode;
    use super::Scancode;

    pub(in super) fn parse(scancode: Scancode) -> Option<Keycode> {
        if scancode.extended {
            parse_extended(scancode.code)
        } else {
            parse_standard(scancode.code)
        }
    }

    fn parse_standard(code: u8) -> Option<Keycode> {
        match code {
            0x01 => Some(F9),
            0x03 => Some(F5),
            0x04 => Some(F3),
            0x05 => Some(F1),
            0x06 => Some(F2),
            0x07 => Some(F12),
            0x09 => Some(F10),
            0x0A => Some(F8),
            0x0B => Some(F6),
            0x0C => Some(F4),
            0x0D => Some(TAB),
            0x0E => Some(BACK_TICK),
            0x11 => Some(LEFT_ALT),
            0x12 => Some(LEFT_SHIFT),
            0x14 => Some(LEFT_CONTROL),
            0x15 => Some(Q),
            0x16 => Some(KEY_1),
            0x1A => Some(Z),
            0x1B => Some(S),
            0x1C => Some(A),
            0x1D => Some(W),
            0x1E => Some(KEY_2),
            0x21 => Some(C),
            0x22 => Some(X),
            0x23 => Some(D),
            0x24 => Some(E),
            0x25 => Some(KEY_4),
            0x26 => Some(KEY_3),
            0x29 => Some(SPACE),
            0x2A => Some(V),
            0x2B => Some(F),
            0x2C => Some(T),
            0x2D => Some(R),
            0x2E => Some(KEY_5),
            0x31 => Some(N),
            0x32 => Some(B),
            0x33 => Some(H),
            0x34 => Some(G),
            0x35 => Some(Y),
            0x36 => Some(KEY_6),
            0x3A => Some(M),
            0x3B => Some(J),
            0x3C => Some(U),
            0x3D => Some(KEY_7),
            0x3E => Some(KEY_8),
            0x41 => Some(COMMA),
            0x42 => Some(K),
            0x43 => Some(I),
            0x44 => Some(O),
            0x45 => Some(KEY_0),
            0x46 => Some(KEY_9),
            0x49 => Some(PERIOD),
            0x4A => Some(FORWARD_SLASH),
            0x4B => Some(L),
            0x4C => Some(SEMI_COLON),
            0x4D => Some(P),
            0x4E => Some(MINUS),
            0x52 => Some(SINGLE_QUOTE),
            0x54 => Some(SQUARE_BRACKET_OPEN),
            0x55 => Some(EQUALS),
            0x58 => Some(CAPS_LOCK),
            0x59 => Some(RIGHT_SHIFT),
            0x5A => Some(ENTER),
            0x5B => Some(SQUARE_BRACKET_CLOSE),
            0x5D => Some(BACK_SLASH),
            0x66 => Some(BACKSPACE),
            0x69 => Some(NUM_PAD_1),
            0x6B => Some(NUM_PAD_4),
            0x6C => Some(NUM_PAD_7),
            0x70 => Some(NUM_PAD_0),
            0x71 => Some(NUM_PAD_PERIOD),
            0x72 => Some(NUM_PAD_2),
            0x73 => Some(NUM_PAD_5),
            0x74 => Some(NUM_PAD_6),
            0x75 => Some(NUM_PAD_8),
            0x76 => Some(ESCAPE),
            0x77 => Some(NUM_LOCK),
            0x78 => Some(F11),
            0x79 => Some(NUM_PAD_PLUS),
            0x7A => Some(NUM_PAD_3),
            0x7B => Some(NUM_PAD_MINUS),
            0x7C => Some(NUM_PAD_ASTERISK),
            0x7D => Some(NUM_PAD_9),
            0x7E => Some(SCROLL_LOCK),
            0x83 => Some(F7),
            _ => None,
        }
    }

    fn parse_extended(code: u8) -> Option<Keycode> {
        match code {
            0x11 => Some(RIGHT_ALT),
            0x14 => Some(RIGHT_CONTROL),
            0x4A => Some(NUM_PAD_FORWARD_SLASH),
            0x5A => Some(NUM_PAD_ENTER),
            0x69 => Some(END),
            0x6B => Some(LEFT_ARROW),
            0x6C => Some(HOME),
            0x70 => Some(INSERT),
            0x71 => Some(DELETE),
            0x72 => Some(DOWN_ARROW),
            0x74 => Some(RIGHT_ARROW),
            0x75 => Some(UP_ARROW),
            0x7A => Some(PAGE_DOWN),
            0x7D => Some(PAGE_UP),
            _ => None,
        }
    }
}
