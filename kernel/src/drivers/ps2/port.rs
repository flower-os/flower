use crate::drivers::ps2;
use crate::drivers::ps2::{Device, Controller, Keyboard, Mouse};
use crossbeam::queue::ArrayQueue;

lazy_static! {
    pub(in super) static ref KEYBOARD_INPUT: InputQueue = InputQueue::new();
    pub(in super) static ref MOUSE_INPUT: InputQueue = InputQueue::new();
}

pub fn detect() -> ps2::Result<()> {
    if !Keyboard::test()? {
        warn!("ps2c: keyboard port not available");
    }

    let dual_channel = test_dual_channel()?;
    if !dual_channel {
        warn!("ps2c: controller is not dual channel");
    }

    if !(dual_channel && Mouse::test()?) {
        warn!("ps2c: mouse port not available");
    }

    Ok(())
}

fn test_dual_channel() -> ps2::Result<bool> {
    Mouse::disable()?;

    // If the second port is disabled and supported, `MOUSE_CLOCK_DISABLED` should be set
    let config = Controller::read_config()?;
    if !config.contains(ps2::ConfigFlags::MOUSE_CLOCK_DISABLED) {
        return Ok(false);
    }

    // Temporarily enable the port and read the config again
    Mouse::enable()?;
    let config = Controller::read_config()?;
    Mouse::disable()?;

    // If the second port is enabled and supported, `MOUSE_CLOCK_DISABLED` will now be clear
    Ok(!config.contains(ps2::ConfigFlags::MOUSE_CLOCK_DISABLED))
}

pub struct InputQueue {
    buffer: ArrayQueue<u8>,
}

impl InputQueue {
    fn new() -> InputQueue {
        InputQueue {
            buffer: ArrayQueue::new(64),
        }
    }

    pub fn push(&self, byte: u8) {
        // if we overflow, we can safely ignore. we want to keep old bytes and discard new ones
        let _ = self.buffer.push(byte);
    }

    pub(in super) fn next(&self) -> Option<u8> {
        self.buffer.pop().ok()
    }
}
