//! # Keyboard Driver
//!
//! The keyboard driver handles all keyboard related functionality, intended to support both PS/2 and USB.
//! Currently, only PS/2 support has been implemented through the use of the PS/2 driver.
//!
//! The driver is event based, and events are received through the `read_event` method, which blocks until an event is received.
//! The event contains the keycode pressed, which can be compared to `keymap::codes`, an optional `char`, the type of press, and various modifier flags.
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

// TODO: Redo all examples

pub mod keymap;

use crate::drivers::ps2;

bitflags! {
    pub struct ModifierFlags: u8 {
        /// If a SHIFT modifier is active
        const SHIFT = 1 << 0;
        /// If the NUM_LOCK modifier is active
        const NUM_LOCK = 1 << 1;
        /// If a CAPS_LOCK modifier is active
        const CAPS_LOCK = 1 << 2;
    }
}

bitflags! {
    /// Flags to hold the current keyboard lock states
    pub struct StateFlags: u8 {
        /// If num lock is enabled
        const NUM_LOCK = 1 << 0;
        /// If scroll lock is enabled
        const SCROLL_LOCK = 1 << 1;
        /// If caps lock is enabled
        const CAPS_LOCK = 1 << 2;
        /// If function lock is enabled
        const FUNCTION_LOCK = 1 << 3;
    }
}

impl ModifierFlags {
    /// Creates `ModifierFlags` from the given contained booleans
    ///
    /// # Examples
    ///
    /// ```rust
    /// let modifiers = ModifierFlags::from_modifiers(true, true, true);
    /// assert_eq!(modifiers, ModifierFlags::SHIFT | ModifierFlags::NUM_LOCK | ModifierFlags::CAPS_LOCK);
    /// ```
    fn from_modifiers(shift: bool, num_lock: bool, caps_lock: bool) -> Self {
        let mut flags = ModifierFlags::empty();
        flags.set(ModifierFlags::SHIFT, shift);
        flags.set(ModifierFlags::NUM_LOCK, num_lock);
        flags.set(ModifierFlags::CAPS_LOCK, caps_lock);
        flags
    }
}

/// Mapping from a keycode into a character
pub enum KeyCharMapping {
    /// A key with no character mapping
    Empty,
    /// A key with a constant character mapping
    Single(char),
    /// A key with an alternative character mapping when shift is pressed
    Shifted(char, char),
    /// A key with an alternative character mapping when either CAPS is enabled or shift is pressed
    Capitalized(char, char),
    /// A key that only maps to a character when numlock is disabled
    NumLocked(char),
}

impl KeyCharMapping {
    /// Gets the character for this char based on the given modifiers
    pub fn char(&self, modifiers: ModifierFlags) -> Option<char> {
        use self::KeyCharMapping::*;
        match *self {
            Single(character) => Some(character),
            Shifted(character, shifted) => if modifiers.contains(ModifierFlags::SHIFT) {
                Some(shifted)
            } else {
                Some(character)
            },
            Capitalized(character, capital) => if modifiers.contains(ModifierFlags::CAPS_LOCK) ^ modifiers.contains(ModifierFlags::SHIFT) {
                Some(capital)
            } else {
                Some(character)
            },
            NumLocked(character) if !modifiers.contains(ModifierFlags::NUM_LOCK) => Some(character),
            _ => None,
        }
    }
}

pub type Keycode = u8;

/// Contains data relating to a key press event
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub keycode: Keycode,
    pub char: Option<char>,
    pub kind: KeyEventKind,
    pub modifiers: ModifierFlags,
}

/// The kind of key event that occurred
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum KeyEventKind {
    /// When the key is initially pressed
    Make,
    /// When the key is released
    Break,
    /// When the key is held down, and a repeat is fired
    Repeat,
}

/// Interface to a generic keyboard.
pub trait Keyboard {
    type Error;

    /// Polls the device for a new key state event, or returns `None` if none have occurred since the last poll.
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
    fn read_event(&mut self) -> Result<Option<KeyEvent>, Self::Error>;

    /// Returns `true` if the given keycode is currently being held down
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    ///
    /// if keyboard.is_down(keymap::codes::LEFT_SHIFT) {
    ///     println!("Left shift down");
    /// } else {
    ///     println!("Left shift not down");
    /// }
    /// ```
    fn is_down(&self, keycode: Keycode) -> bool;

    fn num_lock(&self) -> bool;

    fn scroll_lock(&self) -> bool;

    fn caps_lock(&self) -> bool;

    fn function_lock(&self) -> bool;
}

const KEY_STATE_WORD_WIDTH: usize = 8;
/// The amount of words each containing 8 key state bits
const KEY_STATE_LENGTH: usize = 0xFF / KEY_STATE_WORD_WIDTH;

/// Handles interface to a PS/2 keyboard, if available
pub struct Ps2Keyboard {
    /// Bitmap containing state of every key. 8 key states are stored per entry (word)
    key_state_words: [u8; KEY_STATE_LENGTH],
    state: StateFlags,
}

impl Ps2Keyboard {
    /// Creates a new Ps2Keyboard from the given PS/2 device
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let device = drivers::ps2::CONTROLLER.device(drivers::ps2::DevicePort::Keyboard);
    /// let mut keyboard = Ps2Keyboard::new(device);
    /// ```
    pub fn new() -> Self {
        Ps2Keyboard {
            key_state_words: [0; KEY_STATE_LENGTH],
            state: StateFlags::empty(),
        }
    }

    fn on_keyboard_change(&self) -> ps2::Result<()> {
        ps2::Keyboard::set_scanset(ps2::keyboard::Scanset::Two)
    }

    fn create_event(&self, keycode: Keycode, make: bool) -> KeyEvent {
        let shift = self.is_down(keymap::codes::LEFT_SHIFT) || self.is_down(keymap::codes::RIGHT_SHIFT);
        let num_lock = self.state.contains(StateFlags::NUM_LOCK);
        let caps_lock = self.state.contains(StateFlags::CAPS_LOCK);
        let modifiers = ModifierFlags::from_modifiers(shift, num_lock, caps_lock);

        let char = keymap::get_us_qwerty_char(keycode).char(modifiers);

        // If the key was already pressed and make was sent, this is a repeat event
        let kind = match make {
            true if self.is_down(keycode) => KeyEventKind::Repeat,
            true => KeyEventKind::Make,
            false => KeyEventKind::Break,
        };

        KeyEvent { keycode, char, kind, modifiers }
    }

    fn handle_event(&mut self, event: KeyEvent) -> ps2::Result<()> {
        if event.kind == KeyEventKind::Make {
            use self::keymap::codes::*;
            let last_state = self.state.bits();
            match event.keycode {
                SCROLL_LOCK => self.state.toggle(StateFlags::SCROLL_LOCK),
                NUM_LOCK => self.state.toggle(StateFlags::NUM_LOCK),
                CAPS_LOCK => self.state.toggle(StateFlags::CAPS_LOCK),
                ESCAPE if self.is_down(FUNCTION) => self.state.toggle(StateFlags::FUNCTION_LOCK),
                _ => (),
            }
            if self.state.bits() != last_state {
                ps2::Keyboard::set_leds(self.state.into())?;
            }
        }

        let index = event.keycode as usize;
        let bit = 1 << (index % KEY_STATE_WORD_WIDTH);
        let word_index = index / KEY_STATE_WORD_WIDTH;
        if event.kind == KeyEventKind::Make {
            self.key_state_words[word_index] |= bit;
        } else {
            self.key_state_words[word_index] &= !bit;
        }

        Ok(())
    }
}

impl Keyboard for Ps2Keyboard {
    type Error = ps2::Error;

    fn read_event(&mut self) -> ps2::Result<Option<KeyEvent>> {
        if let Some(event) = ps2::Keyboard::next_event() {
            match event {
                ps2::keyboard::Event::Key { key, make } => {
                    let event = self.create_event(key, make);
                    self.handle_event(event)?;
                    return Ok(Some(event));
                }
                ps2::keyboard::Event::BatSuccess => self.on_keyboard_change()?,
                _ => ()
            }
        }

        Ok(None)
    }

    fn is_down(&self, keycode: Keycode) -> bool {
        let index = keycode as usize;
        let word = *self.key_state_words.get(index / KEY_STATE_WORD_WIDTH).unwrap_or(&0);
        let bit_index = index % KEY_STATE_WORD_WIDTH;
        ((word >> bit_index) & 1) != 0
    }

    fn num_lock(&self) -> bool { self.state.contains(StateFlags::NUM_LOCK) }

    fn scroll_lock(&self) -> bool { self.state.contains(StateFlags::SCROLL_LOCK) }

    fn caps_lock(&self) -> bool { self.state.contains(StateFlags::CAPS_LOCK) }

    fn function_lock(&self) -> bool { self.state.contains(StateFlags::FUNCTION_LOCK) }
}

impl From<StateFlags> for ps2::keyboard::LedFlags {
    fn from(state: StateFlags) -> Self {
        use ps2::keyboard::LedFlags;
        let mut flags = LedFlags::empty();
        if state.contains(StateFlags::SCROLL_LOCK) {
            flags.insert(LedFlags::SCROLL_LOCK);
        }
        if state.contains(StateFlags::NUM_LOCK) {
            flags.insert(LedFlags::NUMBER_LOCK);
        }
        if state.contains(StateFlags::CAPS_LOCK) {
            flags.insert(LedFlags::CAPS_LOCK);
        }
        flags
    }
}
