use crate::drivers::ps2;
use crate::drivers::ps2::Device;
use crate::drivers::ps2::io::CommandIo;

use spin::Mutex;

static SCANCODE_PARSER: Mutex<ScancodeParser> = Mutex::new(ScancodeParser::new());

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Scancode {
    pub code: u8,
    pub extended: bool,
    pub make: bool,
}

impl Scancode {
    pub fn new(scancode: u8, extended: bool, make: bool) -> Self {
        Scancode { code: scancode, extended, make }
    }
}

struct ScancodeParser {
    make: bool,
    extended: bool,
}

impl ScancodeParser {
    const fn new() -> ScancodeParser {
        ScancodeParser {
            make: true,
            extended: false,
        }
    }

    fn push(&mut self, byte: u8) -> Option<Scancode> {
        match byte {
            0xE0...0xE1 => self.extended = true,
            0xF0 => self.make = false,
            _ => return Some(Scancode::new(byte, self.extended, self.make)),
        }
        None
    }

    fn reset(&mut self) {
        self.make = true;
        self.extended = false;
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
        ps2::io::read_blocking(&ps2::io::DATA_PORT).ok_or(ps2::Error::ExpectedResponse)
    }
}

impl ps2::Device for Keyboard {
    #[inline]
    fn enable() -> ps2::Result<()> { ps2::Controller::enable_keyboard() }

    #[inline]
    fn disable() -> ps2::Result<()> { ps2::Controller::disable_keyboard() }

    #[inline]
    fn test() -> ps2::Result<bool> { ps2::Controller::test_keyboard() }

    #[inline]
    fn input_queue() -> &'static ps2::port::InputQueue { &*ps2::port::KEYBOARD_INPUT_QUEUE }
}

impl Keyboard {
    // TODO: possibly handle invalid scancodes here?
    pub fn next_scancode() -> Option<Scancode> {
        let input_queue = Self::input_queue();
        let mut scancode_parser = SCANCODE_PARSER.lock();

        while let Some(byte) = input_queue.next() {
            if let Some(scancode) = scancode_parser.push(byte) {
                scancode_parser.reset();
                return Some(scancode);
            }
        }

        None
    }

    #[inline]
    pub fn send_repeat_events() -> ps2::Result<()> { Self::send(0xF7) }

    #[inline]
    pub fn send_make_release_events() -> ps2::Result<()> { Self::send(0xF8) }

    #[inline]
    pub fn send_make_events() -> ps2::Result<()> { Self::send(0xF9) }

    #[inline]
    pub fn send_all_events() -> ps2::Result<()> { Self::send(0xFA) }

    #[inline]
    pub fn set_leds(flags: ps2::keyboard::LedFlags) -> ps2::Result<()> {
        Self::send_data(0xED, flags.bits())
    }

    #[inline]
    pub fn set_typematic_options(flags: u8) -> ps2::Result<()> {
        Self::send_data(0xF3, flags)
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
