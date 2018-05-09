//! # Keyboard Driver
//!
//! The keyboard driver handles all keyboard related functionality, intended to support both PS/2
//! and USB. Currently, only PS/2 support has been implemented through the use of the PS/2 driver.
//!
//! The driver is event based, and events are received through the `read_event` method, which blocks
//! until an event is received.  The event contains the keycode pressed, which can be compared to
//! `keymap::codes`, an optional `char`, the type of press, and various modifier flags.
//!
//! # Examples
//!
//! ```rust,no_run
//! let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
//! let mut keyboard = Ps2Keyboard::new(device);
//!
//! keyboard.enable()?;
//! loop {
//!     let event = keyboard.read_event()?;
//!     handle_event(event);
//! }
//! ```

pub mod keymap;

use core::convert::From;

use drivers::ps2::{self, Device, DeviceState};
use drivers::ps2::io::Ps2Error;
use drivers::ps2::io::commands::{DeviceCommand, DeviceDataCommand};

bitflags! {
    pub struct ModifierFlags: u8 {
        /// If a CTRL modifier is active
        const CTRL = 1;
        /// If an ALT modifier is active
        const ALT = 1 << 1;
        /// If a SHIFT modifier is active
        const SHIFT = 1 << 2;
    }
}

impl ModifierFlags {
    /// Creates `ModifierFlags` from the given contained booleans
    ///
    /// # Examples
    ///
    /// ```rust
    /// let modifiers = ModifierFlags::from_modifiers(true, true, true);
    /// assert_eq!(modifiers, ModifierFlags::CTRL | ModifierFlags::ALT | ModifierFlags::SHIFT);
    /// ```
    fn from_modifiers(ctrl: bool, alt: bool, shift: bool) -> Self {
        let mut flags = ModifierFlags::empty();
        flags.set(ModifierFlags::CTRL, ctrl);
        flags.set(ModifierFlags::ALT, alt);
        flags.set(ModifierFlags::SHIFT, shift);
        flags
    }
}

/// Contains data relating to a key press event
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    pub keycode: u8,
    pub char: Option<char>,
    pub event_type: KeyEventType,
    pub modifiers: ModifierFlags,
}

/// The type of key event that occurred
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyEventType {
    /// When the key is initially pressed
    Make,
    /// When the key is released
    Break,
    /// When the key is held down, and a repeat is fired
    Repeat,
}

/// An error for a PS/2 keyboard
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ps2KeyboardError {
    /// If an error occurred while reading from PS/2
    ReadError(Ps2Error),
    /// If the keyboard is disabled and cannot be used
    KeyboardDisabled,
    /// If enabling the keyboard fails
    KeyboardEnableFailed,
    /// If setting the scancode fails
    ScancodeSetFailed,
    /// If enabling scanning fails
    ScanningEnableFailed,
}

/// Interface to a generic keyboard.
pub trait Keyboard {
    type Error;

    /// Enables this keyboard, setting it up before use.
    ///
    /// # Note
    ///
    /// The keyboard should not be accessed while not enabled!
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    ///
    /// match keyboard.enable() {
    ///     Ok(_) => println!("Keyboard successfully enabled"),
    ///     Err(err) => println!("Keyboard enable failed with error: {:?}", err),
    /// }
    /// ```
    fn enable(&mut self) -> Result<(), Self::Error>;

    /// Disables this keyboard, making use unavailable.
    ///
    /// # Note
    ///
    /// Until `enable` is called again, this keyboard should not be used
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    ///
    /// match keyboard.disable() {
    ///     Ok(_) => println!("Keyboard successfully disabled"),
    ///     Err(err) => println!("Keyboard disable failed with error: {:?}", err),
    /// }
    /// ```
    fn disable(&mut self) -> Result<(), Self::Error>;

    /// Polls the device for a new key state event, or returns `None` if none have occurred since
    /// the last poll.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    ///
    /// if let Some(event) = keyboard.read_event()? {
    ///     println!("Event occurred for char: {}", event.char.unwrap_or(' '));
    /// }
    /// ```
    // TODO: This should eventually use interrupts and hold a queue
    fn read_event(&mut self) -> Result<Option<KeyEvent>, Self::Error>;

    /// Returns `true` if the given keycode is currently being pressed
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    ///
    /// if keyboard.pressed(keymap::codes::LEFT_SHIFT) {
    ///     println!("Left shift pressed");
    /// } else {
    ///     println!("Left shift not pressed");
    /// }
    /// ```
    fn pressed(&self, keycode: u8) -> bool;
}

/// Handles interface to a PS/2 keyboard, if available
pub struct Ps2Keyboard<'a> {
    device: &'a mut Device,
    key_states: [bool; 0xFF],
}

impl<'a> Ps2Keyboard<'a> {
    /// Creates a new Ps2Keyboard from the given PS/2 device
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    /// ```
    pub fn new(device: &'a mut Device) -> Self {
        Ps2Keyboard {
            device,
            key_states: [false; 0xFF],
        }
    }

    /// Reads a single scancode from this PS/2 keyboard
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    ///
    /// if let Some(scancode) = keyboard.read_scancode()? {
    ///     print!(scancode);
    /// }
    /// ```
    fn read_scancode(&self) -> Result<Option<Ps2Scancode>, Ps2KeyboardError> {
        use ps2::io;

        if self.device.state == DeviceState::Enabled {
            if io::can_read()? && io::can_read_keyboard()? {
                let mut make = true;
                let mut extended = false;

                // Get all scancode modifiers, and return when the actual scancode is received
                let scancode = (io::DATA_PORT.with_lock(|mut data_port| loop {
                    let data = io::read(&mut data_port)?;
                    match data {
                        0xE0...0xE1 => extended = true,
                        0xF0 => make = false,
                        _ => {
                            break Ok(data);
                        }
                    }
                }): Result<u8, io::Ps2Error>)?;

                // If scancode is present, return it with modifiers
                return Ok(if scancode != 0 {
                    Some(Ps2Scancode::new(scancode, extended, make))
                } else {
                    None
                });
            }
            Ok(None)
        } else {
            Err(Ps2KeyboardError::KeyboardDisabled)
        }
    }

    /// Creates a [KeyEvent] from the given scancode and key state
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let scancode = Ps2Scancode::new(0x15, false, true);
    /// let event = keyboard.create_event(&scancode).unwrap();
    /// assert_eq!(event.keycode, keymap::codes::Q);
    /// assert_eq!(event.char, Some('q'));
    /// assert_eq!(event.event_type, KeyEventType::Make);
    /// ```
    fn create_event(&self, scancode: &Ps2Scancode) -> Option<KeyEvent> {
        let ctrl = self.pressed(keymap::codes::LEFT_CONTROL) ||
            self.pressed(keymap::codes::RIGHT_CONTROL);
        let alt = self.pressed(keymap::codes::LEFT_ALT) || self.pressed(keymap::codes::RIGHT_ALT);
        let shift = self.pressed(keymap::codes::LEFT_SHIFT) ||
            self.pressed(keymap::codes::RIGHT_SHIFT);
        let modifiers = ModifierFlags::from_modifiers(ctrl, alt, shift);

        if let Some(keycode) = scancode.keycode() {
            let char = keymap::get_us_qwerty_char(keycode).map(|chars| if shift {
                chars.1
            } else {
                chars.0
            });

            // If the key was already pressed and make was sent, this is a repeat event
#[allow(match_bool)] // Looks nicer
            let event_type = match scancode.make {
                true if self.pressed(keycode) => KeyEventType::Repeat,
                true => KeyEventType::Make,
                false => KeyEventType::Break,
            };

            return Some(KeyEvent {
                keycode,
                char,
                event_type,
                modifiers,
            });
        }

        None
    }
}

impl<'a> Keyboard for Ps2Keyboard<'a> {
    type Error = Ps2KeyboardError;

    fn enable(&mut self) -> Result<(), Ps2KeyboardError> {
        self.device.enable()?;

        if self.device.state != DeviceState::Enabled {
            return Err(Ps2KeyboardError::KeyboardEnableFailed);
        }

        if self.device.command_data(DeviceDataCommand::SetScancode, 2)? != ps2::ACK {
            return Err(Ps2KeyboardError::ScancodeSetFailed);
        }

        if self.device.command(DeviceCommand::EnableScanning)? != ps2::ACK {
            return Err(Ps2KeyboardError::ScanningEnableFailed);
        }

        Ok(())
    }

    fn disable(&mut self) -> Result<(), Ps2KeyboardError> {
        self.device.disable()?;

        Ok(())
    }

    fn read_event(&mut self) -> Result<Option<KeyEvent>, Self::Error> {
        let event = self.read_scancode()?.and_then(|scancode| {
            let event = self.create_event(&scancode);
            if event.is_some() {
                self.key_states[event.unwrap().keycode as usize] = scancode.make;
            }
            event
        });
        Ok(event)
    }

    fn pressed(&self, keycode: u8) -> bool {
        *self.key_states.get(keycode as usize).unwrap_or(&false)
    }
}

/// Represents a PS/2 scancode received from the device
struct Ps2Scancode {
    pub code: u8,
    pub extended: bool,
    pub make: bool,
}

impl Ps2Scancode {
    /// Constructs a new [Ps2Scancode]
    fn new(scancode: u8, extended: bool, make: bool) -> Self {
        Ps2Scancode {
            code: scancode,
            extended,
            make,
        }
    }

    /// Gets the Flower keycode for this scancode
    ///
    /// # Examples
    ///
    /// ```rust
    /// let scancode = Ps2Scancode::new(0x01, false, true);
    /// assert_eq!(scancode.keycode(), Some(keymap::codes::KEY_F9));
    /// ```
    fn keycode(&self) -> Option<u8> {
        if self.extended {
            keymap::get_extended_code_ps2_set_2(self.code)
        } else {
            keymap::get_code_ps2_set_2(self.code)
        }
    }
}

impl From<Ps2Error> for Ps2KeyboardError {
    fn from(error: Ps2Error) -> Self {
        Ps2KeyboardError::ReadError(error)
    }
}
