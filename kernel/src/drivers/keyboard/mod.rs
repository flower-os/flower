//! # Keyboard Driver
//!
//! The keyboard driver handles all keyboard related functionality, intended to support both PS/2 and USB.
//! Currently, only PS/2 support has been implemented through the use of the PS/2 driver.
//!
//! The driver is event based, and events are received through the `read_event` method, which blocks until an event is received.
//! The event contains the keycode pressed, which can be compared to `keymap::codes`, an optional `char`, the type of press, and various modifier flags.

pub mod keymap;

use core::convert::From;

use drivers::ps2::{self, Device, DeviceState};
use drivers::ps2::io::Ps2Error;
use drivers::ps2::io::commands::{DeviceCommand, DeviceDataCommand};

/// Interface to a generic keyboard.
pub trait Keyboard {
    type Error;

    /// Enables this keyboard, setting it up before use.
    ///
    /// # Note
    ///
    /// The keyboard should not be accessed while not enabled!
    fn enable(&mut self) -> Result<(), Self::Error>;

    /// Disables this keyboard, making use unavailable.
    ///
    /// # Note
    ///
    /// Until `enable` is called again, this keyboard should not be used
    fn disable(&mut self) -> Result<(), Self::Error>;

    /// Polls the device for state events.
    /// TODO: This should eventually use interrupts and hold a queue
    fn read_event(&mut self) -> Result<KeyEvent, Self::Error>;

    /// Returns true if the given keycode is currently being pressed
    fn pressed(&self, keycode: u8) -> bool;
}

/// Contains data relating to a key press event.
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    pub keycode: u8,
    pub char: Option<char>,
    pub event_type: KeyEventType,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

/// The type of key event occurring
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyEventType {
    Make,
    Break,
    Repeat,
}

/// An error for a PS/2 keyboard
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ps2KeyboardError {
    ReadError(Ps2Error),
    KeyboardDisabled,
    ScancodeNotSet(u8),
    ScanningNotEnabled,
    DataUnavailable,
}

/// Handles interface to a PS/2 keyboard, if available
pub struct Ps2Keyboard<'a> {
    device: &'a mut Device,
    key_states: [bool; 0xFF],
}

impl<'a> Ps2Keyboard<'a> {
    pub fn new(device: &'a mut Device) -> Self {
        Ps2Keyboard {
            device,
            key_states: [false; 0xFF],
        }
    }

    /// Reads a single scancode from this
    fn read_scancode(&self) -> Result<Ps2Scancode, Ps2KeyboardError> {
        self.check_enabled()?;

        if ps2::io::can_read()? && ps2::io::can_read_keyboard()? {
            // Check scancode present
            let scancode = ps2::io::read(&ps2::io::DATA_PORT)?;

            // Check not extended scancode (ignored)
            if scancode != 0 && scancode != 0xE0 && scancode != 0xE1 {
                // If not key release (break) code
                if scancode < 0x80 {
                    return Ok(Ps2Scancode::new(scancode, true));
                } else {
                    return Ok(Ps2Scancode::new(scancode - 0x80, false));
                }
            }
        }

        Err(Ps2KeyboardError::DataUnavailable)
    }

    /// Checks if this keyboard is enabled, and returns an error if not
    fn check_enabled(&self) -> Result<(), Ps2KeyboardError> {
        if self.device.state == DeviceState::Enabled {
            Ok(())
        } else {
            Err(Ps2KeyboardError::KeyboardDisabled)
        }
    }

    /// Creates a [KeyEvent] from the given scancode and key state
    fn create_event(&self, scancode: &Ps2Scancode) -> KeyEvent {
        let ctrl = self.pressed(keymap::codes::LEFT_CONTROL) || self.pressed(keymap::codes::RIGHT_CONTROL);
        let alt = self.pressed(keymap::codes::LEFT_ALT) || self.pressed(keymap::codes::RIGHT_ALT);
        let shift = self.pressed(keymap::codes::LEFT_SHIFT) || self.pressed(keymap::codes::RIGHT_SHIFT);

        let keycode = *keymap::PS2_SET_1.get(scancode.code as usize).unwrap_or(&0x0);
        let char = keymap::get_us_qwerty_char(keycode)
            .map(|chars| if shift {
                chars.1
            } else {
                chars.0
            });

        // If the key was already pressed and make was sent, this is a repeat event
        let event_type = match scancode.make {
            true if self.pressed(keycode) => KeyEventType::Repeat,
            true => KeyEventType::Make,
            false => KeyEventType::Break,
        };

        KeyEvent { keycode, char, event_type, ctrl, alt, shift }
    }
}

impl<'a> Keyboard for Ps2Keyboard<'a> {
    type Error = Ps2KeyboardError;

    fn enable(&mut self) -> Result<(), Ps2KeyboardError> {
        self.device.enable()?;

        if self.device.command_data(DeviceDataCommand::SetScancode, 1)? != ps2::ACK {
            return Err(Ps2KeyboardError::ScancodeNotSet(1));
        }

        if self.device.command(DeviceCommand::EnableScanning)? != ps2::ACK {
            return Err(Ps2KeyboardError::ScanningNotEnabled);
        }

        Ok(())
    }

    fn disable(&mut self) -> Result<(), Ps2KeyboardError> {
        self.device.disable()?;

        Ok(())
    }

    fn read_event(&mut self) -> Result<KeyEvent, Self::Error> {
        match self.read_scancode() {
            Ok(scancode) => {
                let event = self.create_event(&scancode);
                self.key_states[event.keycode as usize] = scancode.make;
                Ok(event)
            }
            Err(error) => Err(error),
        }
    }

    fn pressed(&self, keycode: u8) -> bool {
        *self.key_states.get(keycode as usize).unwrap_or(&false)
    }
}

/// Represents a PS/2 scancode received from the device
struct Ps2Scancode {
    pub code: u8,
    pub make: bool,
}

impl Ps2Scancode {
    fn new(scancode: u8, make: bool) -> Self {
        Ps2Scancode {
            code: scancode,
            make,
        }
    }
}

impl From<Ps2Error> for Ps2KeyboardError {
    fn from(error: Ps2Error) -> Self {
        Ps2KeyboardError::ReadError(error)
    }
}
