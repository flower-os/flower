// TODO: consider API for accessing devices

use crate::drivers::ps2;
use crate::drivers::ps2::io::CommandIo;
use crate::drivers::ps2::Controller;

pub const RESEND: u8 = 0xFE;
pub const ACK: u8 = 0xFA;
pub const ECHO: u8 = 0xEE;

/// The amount of iterations to resend a device command before returning an error
pub const COMMAND_RETRIES: usize = 8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DeviceKind {
    Unknown,
    Keyboard(KeyboardKind),
    Mouse(MouseKind),
}

impl DeviceKind {
    pub fn parse(identity: u8, keyboard: bool) -> DeviceKind {
        use self::DeviceKind::*;
        match identity {
            0x41 | 0xC1 if keyboard => Keyboard(KeyboardKind::Mf2TranslatedKeyboard),
            0x83 if keyboard => Keyboard(KeyboardKind::Mf2Keyboard),
            _ if keyboard => Keyboard(KeyboardKind::Unknown),

            0x00 if !keyboard => Mouse(MouseKind::Mouse),
            0x03 if !keyboard => Mouse(MouseKind::MouseWithScrollWheel),
            0x04 if !keyboard => Mouse(MouseKind::FiveButtonMouse),

            _ => Unknown,
        }
    }

    pub fn is_keyboard(&self) -> bool {
        match self {
            DeviceKind::Keyboard(_) => true,
            _ => false,
        }
    }

    pub fn is_mouse(&self) -> bool {
        match self {
            DeviceKind::Mouse(_) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyboardKind {
    Unknown,
    TranslatedAtKeyboard,
    Mf2Keyboard,
    Mf2TranslatedKeyboard,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MouseKind {
    Mouse,
    MouseWithScrollWheel,
    FiveButtonMouse,
}

pub trait Device: CommandIo + Sized {
    fn enable() -> ps2::Result<()>;

    fn disable() -> ps2::Result<()>;

    fn test() -> ps2::Result<bool>;

    #[inline]
    fn set_defaults() -> ps2::Result<()> { Self::send(0xF6) }

    #[inline]
    fn reset() -> ps2::Result<()> { Self::send(0xFF) }

    #[inline]
    fn identify() -> ps2::Result<DeviceKind> {
        Self::send(0xF2)?;

        let mut identity = Self::read().ok();
        let mut keyboard = false;

        // If we receive 0xAB, we know this is a keyboard and can expect to receive another byte
        if identity == Some(0xAB) {
            keyboard = true;
            identity = Self::read().ok();
        }

        match identity {
            Some(identity) => Ok(DeviceKind::parse(identity, keyboard)),
            // If no response is returned, it must be a translated AT keyboard
            None => Ok(DeviceKind::Keyboard(KeyboardKind::TranslatedAtKeyboard)),
        }
    }

    #[inline]
    fn echo() -> ps2::Result<u8> {
        Self::send(0xEE)?;
        Self::read()
    }

    #[inline]
    fn reset_echo() -> ps2::Result<()> { Self::send(0xEC) }
}

pub(in crate::drivers::ps2) fn send_raw_device_command(command: u8, mouse_port: bool) -> ps2::Result<()> {
    for _ in 0..COMMAND_RETRIES {
        if mouse_port {
            Controller::write_command_mouse()?;
        }

        ps2::io::flush_output();
        ps2::io::write_blocking(&ps2::io::DATA_PORT, command);
        match ps2::io::read_blocking(&ps2::io::DATA_PORT) {
            Some(ACK) | Some(ECHO) => return Ok(()),
            Some(RESEND) => continue,
            Some(unknown) => return Err(ps2::Error::UnexpectedResponse(unknown)),
            None => return Err(ps2::Error::ExpectedResponse),
        }
    }

    trace!("ps2c: exceeded {} retries while sending command {:X}", COMMAND_RETRIES, command);
    Err(ps2::Error::RetriesExceeded)
}

pub(in crate::drivers::ps2) fn send_raw_device_command_data(command: u8, data: u8, mouse_port: bool) -> ps2::Result<()> {
    match send_raw_device_command(command, mouse_port) {
        Ok(_) => {
            ps2::io::write_blocking(&ps2::io::DATA_PORT, data);
            match ps2::io::read_blocking(&ps2::io::DATA_PORT) {
                Some(ACK) => return Ok(()),
                Some(unknown) => return Err(ps2::Error::UnexpectedResponse(unknown)),
                None => return Err(ps2::Error::ExpectedResponse),
            }
        }
        result => result
    }
}
