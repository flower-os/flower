pub mod io;
mod port;
mod device;
pub mod keyboard;
pub mod mouse;

use crate::interrupts::{self, Irq};
use core::{option, result};

pub use self::device::Device;
pub use self::keyboard::Keyboard;
pub use self::mouse::Mouse;

use crate::drivers::ps2::io::CommandIo;

pub const CONTROLLER_TEST_SUCCESS: u8 = 0x55;

bitflags! {
    pub struct ConfigFlags: u8 {
        /// Whether interrupts for Port 1 are enabled
        const KEYBOARD_INTERRUPT = 1 << 0;
        /// Whether interrupts for Port 2 are enabled
        const MOUSE_INTERRUPT = 1 << 1;
        /// Whether the clock for Port 1 is disabled
        const KEYBOARD_CLOCK_DISABLED = 1 << 4;
        /// Whether the clock for Port 2 is disabled
        const MOUSE_CLOCK_DISABLED = 1 << 5;
        /// Whether the controller will transform scan set 2 to scan set 1
        const KEYBOARD_TRANSLATION = 1 << 6;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    RetriesExceeded,
    UnexpectedResponse(u8),
    ExpectedResponse,
    ControllerTestFailed(u8),
}

impl From<option::NoneError> for Error {
    fn from(_: option::NoneError) -> Self { Error::ExpectedResponse }
}

pub type Result<T> = result::Result<T, Error>;

pub struct Controller;

impl CommandIo for Controller {
    fn send(command: u8) -> Result<()> {
        io::flush_output();
        io::write(&io::CONTROLLER_PORT, command);
        Ok(())
    }

    fn send_data(command: u8, data: u8) -> Result<()> {
        io::flush_output();
        io::write(&io::CONTROLLER_PORT, command);
        io::write(&io::DATA_PORT, data);
        Ok(())
    }

    fn read() -> Result<u8> {
        io::read(&io::DATA_PORT).ok_or(Error::ExpectedResponse)
    }
}

impl Controller {
    #[inline]
    pub fn disable_mouse() -> Result<()> { Self::send(0xA7) }

    #[inline]
    pub fn enable_mouse() -> Result<()> { Self::send(0xA8) }

    #[inline]
    pub fn disable_keyboard() -> Result<()> { Self::send(0xAD) }

    #[inline]
    pub fn enable_keyboard() -> Result<()> { Self::send(0xAE) }

    #[inline]
    pub fn write_command_mouse() -> Result<()> { Self::send(0xD4) }

    #[inline]
    pub fn read_config() -> Result<ConfigFlags> {
        Self::send(0x20)?;
        Ok(ConfigFlags::from_bits_truncate(Self::read()?))
    }

    #[inline]
    pub fn test_controller() -> Result<()> {
        Self::send(0xAA)?;
        match Self::read()? {
            CONTROLLER_TEST_SUCCESS => Ok(()),
            result => Err(Error::ControllerTestFailed(result))
        }
    }

    #[inline]
    pub fn test_keyboard() -> Result<bool> {
        Self::send(0xAB)?;
        Ok(Self::read()? == 0x0)
    }

    #[inline]
    pub fn test_mouse() -> Result<bool> {
        Self::send(0xA9)?;
        Ok(Self::read()? == 0x0)
    }

    #[inline]
    pub fn write_config(config: ConfigFlags) -> Result<()> {
        Self::send_data(0x60, config.bits)
    }
}

pub fn initialize() -> Result<()> {
    // TODO: Check if controller is present in FADT

    info!("ps2c: initializing");

    // Disable both ports and flush the output such that we do not get any interference during init
    Controller::disable_keyboard()?;
    Controller::disable_mouse()?;

    io::flush_output();

    debug!("ps2c: disabled devices");

    initialize_config()?;
    debug!("ps2c: controller configured");

    Controller::test_controller()?;
    debug!("ps2c: controller test passed");

    port::detect()?;
    debug!("ps2c: tested ports");

    Keyboard::enable()?;
    Mouse::enable()?;

    Keyboard::reset()?;
    Mouse::reset()?;

    interrupts::listen(Irq::Ps2Keyboard, interrupt_keyboard);
    interrupts::listen(Irq::Ps2Mouse, interrupt_mouse);

    enable_interrupts()?;

    io::flush_output();

    // Make sure nothing got left in the output buffer during initialization
    io::flush_output();

    Ok(())
}

fn interrupt_keyboard() {
    if let Some(byte) = io::read(&io::DATA_PORT) {
        port::KEYBOARD_INPUT.push(byte);
    }
}

fn interrupt_mouse() {
    if let Some(byte) = io::read(&io::DATA_PORT) {
        port::MOUSE_INPUT.push(byte);
    }
}

fn initialize_config() -> Result<()> {
    let mut config = Controller::read_config()?;

    // Set all required config flags
    config.set(ConfigFlags::KEYBOARD_INTERRUPT, false);
    config.set(ConfigFlags::MOUSE_INTERRUPT, false);
    config.set(ConfigFlags::KEYBOARD_TRANSLATION, false);

    // Write the updated config back to the controller
    Controller::write_config(config)?;

    Ok(())
}

fn enable_interrupts() -> Result<()> {
    let mut config = Controller::read_config()?;
    config.set(ConfigFlags::KEYBOARD_INTERRUPT, true);
    config.set(ConfigFlags::MOUSE_INTERRUPT, true);
    Controller::write_config(config)?;

    Ok(())
}
