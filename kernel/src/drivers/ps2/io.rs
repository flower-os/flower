pub mod commands {
    use super::*;

    /// Represents a PS2 controller command without a return value
    #[allow(dead_code)]
    // Dead variants for completeness
    #[derive(Copy, Clone, Debug)]
    #[repr(u8)]
    pub enum ControllerCommand {
        DisablePort2 = 0xA7,
        EnablePort2 = 0xA8,
        DisablePort1 = 0xAD,
        EnablePort1 = 0xAE,
        WriteInputPort2 = 0xD4,
    }

    /// Represents a PS2 controller command with a return value
    #[allow(dead_code)]
    // Dead variants for completeness
    #[derive(Copy, Clone, Debug)]
    #[repr(u8)]
    pub enum ControllerReturnCommand {
        ReadConfig = 0x20,
        TestController = 0xAA,
        TestPort1 = 0xAB,
        TestPort2 = 0xA9,
        IdentifyDevice = 0xF2,
    }

    /// Represents a PS2 controller command with a data value
    #[allow(dead_code)]
    // Dead variants for completeness
    #[derive(Copy, Clone, Debug)]
    #[repr(u8)]
    pub enum ControllerDataCommand {
        WriteConfig = 0x60,
    }

    /// Represents a PS2 device command without data
    #[allow(dead_code)]
    // Dead variants for completeness
    #[derive(Copy, Clone, Debug)]
    #[repr(u8)]
    pub enum DeviceCommand {
        EnableScanning = 0xF4,
        DisableScanning = 0xF5,
        SetDefaults = 0xF6,
        Reset = 0xFF,
    }

    /// Represents a PS2 device command with additional data
    #[allow(dead_code)]
    // Dead variants for completeness
    #[derive(Copy, Clone, Debug)]
    #[repr(u8)]
    pub enum DeviceDataCommand {
        SetScancode = 0xF0,
    }

    /// Sends a controller command without a return
    pub fn send(cmd: ControllerCommand) -> Result<(), Ps2Error> {
        write(&mut COMMAND_PORT.lock(), cmd as u8)
    }

    /// Sends a controller command with data and without a return
    pub fn send_data(cmd: ControllerDataCommand, data: u8) -> Result<(), Ps2Error> {
        let mut command_port = COMMAND_PORT.lock();
        let mut data_port = DATA_PORT.lock();

        write(&mut command_port, cmd as u8)?;
        write(&mut data_port, data)?;

        Ok(())
    }

    /// Sends a controller command with a return
    pub fn send_ret(cmd: ControllerReturnCommand) -> Result<u8, Ps2Error> {
        let mut command_port = COMMAND_PORT.lock();
        let mut data_port = DATA_PORT.lock();

        write(&mut command_port, cmd as u8)?;
        read(&mut data_port)
    }
}

use io::{Port, SynchronizedPort};

pub static DATA_PORT: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x60) };
pub static STATUS_PORT: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x64) };
pub static COMMAND_PORT: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x64) };

/// The number of iterations before assuming no data to be read. Should be changed to a timeout
/// as of #26
pub const WAIT_TIMEOUT: u32 = 1_000_000;

bitflags! {
    pub struct StatusFlags: u8 {
        /// If the output buffer from the controller is full (data can be read)
        const OUTPUT_FULL = 1;
        /// If the input buffer to the controller is full (data cannot be written)
        const INPUT_FULL = 1 << 1;
        /// If the current output from the controller is from the second port
        const OUTPUT_PORT_2 = 1 << 5;
    }
}

/// Represents an error returned by PS/2
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Ps2Error {
    NoData,
    DeviceUnavailable,
}

/// Writes to the given port, or waits until available
pub fn write(port: &mut Port<u8>, value: u8) -> Result<(), Ps2Error> {
    loop {
        // Check if the input status bit is empty
        if can_write()? {
            port.write(value);
            break;
        }
    }

    Ok(())
}

/// Reads from the given port, returning an optional value. `NoData` returned if nothing could be
/// read
pub fn read(port: &mut Port<u8>) -> Result<u8, Ps2Error> {
    for _ in 0..WAIT_TIMEOUT {
        // Check if the output status bit is full
        if can_read()? {
            return Ok(port.read());
        }
    }

    Err(Ps2Error::NoData)
}

/// Flushes the controller's output buffer
pub fn flush_output() -> Result<(), Ps2Error> {
    // Read until the output status bit is empty
    DATA_PORT.with_lock(|mut data_port| {
        while can_read()? {
            data_port.read();
        }

        Ok(())
    })?;

    Ok(())
}

/// Reads from the status port and returns the flags
pub fn read_status() -> Result<StatusFlags, Ps2Error> {
    Ok(StatusFlags::from_bits_truncate(STATUS_PORT.read()))
}

/// Returns true if the write status bit is 0
pub fn can_write() -> Result<bool, Ps2Error> {
    read_status().map(|status| !status.contains(StatusFlags::INPUT_FULL))
}

/// Returns true if the read status bit is 1
pub fn can_read() -> Result<bool, Ps2Error> {
    read_status().map(|status| status.contains(StatusFlags::OUTPUT_FULL))
}

/// Returns true if output port bit is 0, meaning the next data will be read from the keyboard
#[allow(dead_code)] // To be used by drivers interfacing with PS/2
pub fn can_read_keyboard() -> Result<bool, Ps2Error> {
    read_status().map(|status| !status.contains(StatusFlags::OUTPUT_PORT_2))
}

/// Returns true if output port bit is 1, meaning the next data will be read from the mouse
#[allow(dead_code)] // To be used by drivers interfacing with PS/2
pub fn can_read_mouse() -> Result<bool, Ps2Error> {
    read_status().map(|status| status.contains(StatusFlags::OUTPUT_PORT_2))
}
