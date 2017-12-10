use io::IOPort;

pub static DATA_PORT: IOPort = unsafe { IOPort::new(0x60) };
pub static STATUS_PORT: IOPort = unsafe { IOPort::new(0x64) };
pub static COMMAND_PORT: IOPort = unsafe { IOPort::new(0x64) };

pub const WAIT_TIMEOUT: u16 = 1000;

bitflags! {
    pub struct StatusFlags: u8 {
        /// If the output buffer from the controller is full (data can be read)
        const OUTPUT_FULL = 1 << 0;
        /// If the input buffer to the controller is full (data cannot be written)
        const INPUT_FULL = 1 << 1;
        /// If the current output from the controller is from the second port
        const OUTPUT_PORT_2 = 1 << 5;
    }
}

/// Represents a PS2 controller command without a return value
#[allow(dead_code)] // Dead variants for completeness
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ControllerCommand {
    DisablePort2 = 0xA7,
    EnablePort2 = 0xA8,
    DisablePort1 = 0xAD,
    EnablePort1 = 0xAE,
    WriteInputPort2 = 0xD4,
}

/// Represents a PS2 controller command with a return value
#[allow(dead_code)] // Dead variants for completeness
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ControllerReturnCommand {
    ReadConfig = 0x20,
    TestController = 0xAA,
    TestPort1 = 0xAB,
    TestPort2 = 0xA9,
    IdentifyDevice = 0xF2,
}

/// Represents a PS2 controller command with a data value
#[allow(dead_code)] // Dead variants for completeness
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ControllerDataCommand {
    WriteConfig = 0x60,
}

/// Represents a PS2 device command opcode
#[allow(dead_code)] // Dead variants for completeness
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum DeviceCommand {
    EnableScanning = 0xF4,
    DisableScanning = 0xF5,
    SetDefaults = 0xF6,
    SetScancode = 0xF0,
    Reset = 0xFF,
}

/// Represents an error returned by PS/2
#[derive(Copy, Clone, Debug)]
pub enum Ps2Error {
    NoData,
    DeviceUnavailable,
}

/// Sends a controller command without a return
pub fn command(cmd: ControllerCommand) -> Result<(), Ps2Error> {
    write(&COMMAND_PORT, cmd as u8)
}

/// Sends a controller command with data and without a return
pub fn command_data(cmd: ControllerDataCommand, data: u8) -> Result<(), Ps2Error> {
    write(&COMMAND_PORT, cmd as u8)?;
    write(&DATA_PORT, data)?;

    Ok(())
}

/// Sends a controller command with a return
pub fn command_ret(cmd: ControllerReturnCommand) -> Result<u8, Ps2Error> {
    write(&COMMAND_PORT, cmd as u8)?;
    read(&DATA_PORT)
}

/// Writes to the given port, or waits until available
pub fn write(port: &IOPort, value: u8) -> Result<(), Ps2Error> {
    loop {
        // Check if the input status bit is empty
        if can_write()? {
            port.write(value);
            break
        }
    }

    Ok(())
}

/// Reads from the given port, returning an optional value. `NoData` returned if nothing could be read
pub fn read(port: &IOPort) -> Result<u8, Ps2Error> {
    for _i in 0..WAIT_TIMEOUT {
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
    while can_read()? {
        DATA_PORT.read();
    }
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
