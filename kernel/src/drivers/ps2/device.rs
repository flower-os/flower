use crate::drivers::ps2;
use crate::drivers::ps2::io::CommandIo;
use crate::drivers::ps2::Controller;

pub const RESEND: u8 = 0xFE;
pub const ACK: u8 = 0xFA;
pub const ECHO: u8 = 0xEE;

/// The amount of iterations to resend a device command before returning an error
pub const COMMAND_RETRIES: usize = 8;

pub trait Device: CommandIo + Sized {
    fn enable() -> ps2::Result<()>;

    fn disable() -> ps2::Result<()>;

    fn test() -> ps2::Result<bool>;

    #[inline]
    fn reset() -> ps2::Result<()> { Self::send(0xFF) }
}

pub(in crate::drivers::ps2) fn send_raw_device_command(command: u8, mouse_port: bool) -> ps2::Result<()> {
    for _ in 0..COMMAND_RETRIES {
        if mouse_port {
            Controller::write_command_mouse()?;
        }

        ps2::io::flush_output();
        ps2::io::write(&ps2::io::DATA_PORT, command);
        match ps2::io::read(&ps2::io::DATA_PORT) {
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
            ps2::io::write(&ps2::io::DATA_PORT, data);
            match ps2::io::read(&ps2::io::DATA_PORT) {
                Some(ACK) => return Ok(()),
                Some(unknown) => return Err(ps2::Error::UnexpectedResponse(unknown)),
                None => return Err(ps2::Error::ExpectedResponse),
            }
        }
        result => result
    }
}
