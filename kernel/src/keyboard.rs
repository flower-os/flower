use ps2::{self, Ps2Device};
use ps2_io::{self, DeviceCommand};
use keymap::US_QWERTY_2;

/// Interface to keyboard via PS2 protocol
pub trait Keyboard {
    fn enable(&mut self) -> bool;

    fn read_char(&mut self) -> Option<char>;

    fn read_scancode(&mut self) -> Scancode;
}

pub struct Ps2Keyboard<'a> {
    device: &'a mut Ps2Device,
}

impl<'a> Ps2Keyboard<'a> {
    pub fn new(device: &'a mut Ps2Device) -> Self {
        Ps2Keyboard {
            device: device
        }
    }
}

impl<'a> Keyboard for Ps2Keyboard<'a> {
    fn enable(&mut self) -> bool {
        ps2::is_ok(self.device.command_data(DeviceCommand::SetScancode, 2))
            && ps2::is_ok(self.device.command(DeviceCommand::EnableScanning))
    }

    fn read_char(&mut self) -> Option<char> {
        let scancode = self.read_scancode();
        let code = scancode.code as usize;
        if scancode.make && code < US_QWERTY_2.len() {
            Some(US_QWERTY_2[code])
        } else {
            None
        }
    }

    fn read_scancode(&mut self) -> Scancode {
        loop {
            // Check scancode present
            if let Some(scancode) = ps2_io::read_data() {
                // Check not extended (ignored)
                if scancode != 0 && scancode != 0xE0 && scancode != 0xE1 {
                    // If key release (break) code
                    if scancode == 0xF0 {
                        return Scancode::new(ps2_io::read_data().unwrap_or(0) as u32, false);
                    } else {
                        // Return break code
                        return Scancode::new(scancode as u32, true);
                    }
                }
            }
        }
    }
}

/// Represents a scancode
pub struct Scancode {
    pub code: u32,
    pub make: bool,
}

impl Scancode {
    fn new(scancode: u32, make: bool) -> Self {
        Scancode {
            code: scancode,
            make: make,
        }
    }
}
