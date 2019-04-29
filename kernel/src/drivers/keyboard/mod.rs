//! # Keyboard Driver
//!
//! The keyboard driver handles all keyboard related functionality, intended to support both PS/2 and USB.
//! Currently, only PS/2 support has been implemented through the use of the PS/2 driver.
//!
//! The driver is event based, and events are received through the `read_event` method, which will never block.
//! The event contains the keycode pressed, which can be compared to `keymap::codes`, an optional `char`, the type of press, and various modifier flags.
//!
//! # Examples
//!
//! ```rust,no_run
//! loop {
//!     if let Some(event) = ps2_keyboard().read_event()? {
//!         if let Some(char) = event.char {
//!             println!("pressed `{}`", event.char.unwrap());
//!         }
//!     }
//! }
//! ```

pub mod keymap;

use crate::drivers::ps2;
use spin::{Mutex, MutexGuard};

static PS2_KEYBOARD: Mutex<Ps2Keyboard> = Mutex::new(Ps2Keyboard::new());

#[inline]
pub fn ps2_keyboard() -> MutexGuard<'static, Ps2Keyboard> {
    PS2_KEYBOARD.lock()
}

bitflags! {
    pub struct ModifierFlags: u8 {
        /// If shift is pressed
        const SHIFT = 1 << 0;
        /// If num lock is active
        const NUM_LOCK = 1 << 1;
        /// If caps lock is active
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
    /// Resolves the character for this mapping based on the given modifiers
    pub fn resolve(&self, modifiers: ModifierFlags) -> Option<char> {
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

/// Represents an event received from the keyboard.
#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    /// The flower keycode that triggered this event
    pub keycode: Keycode,
    /// The character that this key represents, affected by active modifiers
    /// `None` if a key with no character equivalent such as backspace
    pub char: Option<char>,
    pub kind: KeyEventKind,
    /// Key modifiers active when this event was received
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

/// A generic keyboard
pub trait Keyboard {
    type Error;

    /// Reads a single event from this keyboard. If there are no queued events, `None` is returned.
    /// If no keyboard is present, `None` will also be returned
    ///
    /// This may also return an error depending on the keyboard implementation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// loop {
    ///     if let Some(event) = ps2_keyboard().read_event()? {
    ///         if let Some(char) = event.char {
    ///             println!("pressed `{}`", event.char.unwrap());
    ///         }
    ///     }
    /// }
    /// ```
    fn read_event(&mut self) -> Result<Option<KeyEvent>, Self::Error>;

    /// Returns `true` if the given keycode is currently being held down
    ///
    /// ```rust,no_run
    /// if ps2_keyboard().is_down(keymap::codes::LEFT_SHIFT) {
    ///     println!("Left shift down");
    /// } else {
    ///     println!("Left shift not down");
    /// }
    /// ```
    fn is_down(&self, keycode: Keycode) -> bool;

    /// Returns `true` if num lock is currently active
    fn num_lock(&self) -> bool;

    /// Returns `true` if scroll lock is currently active
    fn scroll_lock(&self) -> bool;

    /// Returns `true` if caps lock is currently active
    fn caps_lock(&self) -> bool;

    /// Returns `true` if function lock is currently active
    fn function_lock(&self) -> bool;
}

const KEY_STATE_WORD_WIDTH: usize = 8;
/// The amount of words each containing 8 key state bits
const KEY_STATE_LENGTH: usize = 0xFF / KEY_STATE_WORD_WIDTH;

/// Provides access to a PS/2 keyboard
pub struct Ps2Keyboard {
    /// Bitmap containing state of every key. 8 key states are stored per word
    key_state_words: [u8; KEY_STATE_LENGTH],
    state: StateFlags,
}

impl Ps2Keyboard {
    const fn new() -> Self {
        Ps2Keyboard {
            key_state_words: [0; KEY_STATE_LENGTH],
            state: StateFlags { bits: 0 },
        }
    }

    fn setup_keyboard(&self) -> ps2::Result<()> {
        ps2::Keyboard::set_scanset(ps2::keyboard::Scanset::Two)
    }

    fn create_event(&self, keycode: Keycode, make: bool) -> KeyEvent {
        let shift = self.is_down(keymap::codes::LEFT_SHIFT) || self.is_down(keymap::codes::RIGHT_SHIFT);
        let num_lock = self.state.contains(StateFlags::NUM_LOCK);
        let caps_lock = self.state.contains(StateFlags::CAPS_LOCK);

        let mut modifiers = ModifierFlags::empty();
        modifiers.set(ModifierFlags::SHIFT, shift);
        modifiers.set(ModifierFlags::NUM_LOCK, num_lock);
        modifiers.set(ModifierFlags::CAPS_LOCK, caps_lock);

        let char = keymap::get_us_qwerty_char(keycode).resolve(modifiers);

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
                ps2::keyboard::Event::BatSuccess => self.setup_keyboard()?,
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
