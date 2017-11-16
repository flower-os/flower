use io::IOPort;

static DATA_PORT: IOPort = IOPort::new(0x60);
static STATUS_PORT: IOPort = IOPort::new(0x64);
static COMMAND_PORT: IOPort = IOPort::new(0x64);

pub const WAIT_TIMEOUT: u16 = 1000;

const OUTPUT_STATUS_BIT: u8 = 1 << 0;
const INPUT_STATUS_BIT: u8 = 1 << 1;

/// Sends a controller command without a return
pub fn command(cmd: ControllerCommand) {
    command_raw(cmd as u8);
}

/// Sends a controller command with data and without a return
pub fn command_data(cmd: ControllerCommand, data: u8) {
    command_raw(cmd as u8);
    write(&DATA_PORT, data);
}

/// Sends a controller command with a return
pub fn command_ret(cmd: ControllerReturnCommand) -> Option<u8> {
    command_raw(cmd as u8);
    read(&DATA_PORT)
}

/// Sends a raw controller command code
fn command_raw(cmd: u8) {
    write(&COMMAND_PORT, cmd);
}

/// Writes the given value to the data port
pub fn write_data(value: u8) {
    write(&DATA_PORT, value);
}

/// Reads from the data port
pub fn read_data() -> Option<u8> {
    read(&DATA_PORT)
}

/// Writes to the given port, or waits until available
pub fn write(port: &IOPort, value: u8) {
    wait_write();
    port.write(value);
}

/// Reads from the given port, returning an optional value. None returned if nothing could be read
pub fn read(port: &IOPort) -> Option<u8> {
    if wait_read() {
        Some(port.read())
    } else {
        None
    }
}

/// Flushes the controller's output buffer
pub fn flush_output() {
    loop {
        // Read until the output status bit is empty
        if check_status(OUTPUT_STATUS_BIT) {
            DATA_PORT.read();
        } else {
            break;
        }
    }
}

/// Waits for the write status bit to empty
fn wait_write() {
    loop {
        // Check if the input status bit is empty
        if !check_status(INPUT_STATUS_BIT) {
            break;
        }
    }
}

/// Waits for the read status bit to equal 1, and returns true if successful
fn wait_read() -> bool {
    for _i in 0..WAIT_TIMEOUT {
        // Check if the output status bit is full
        if check_status(OUTPUT_STATUS_BIT) {
            return true;
        }
    }
    false
}

/// Returns true if the given status bit is 1
fn check_status(bit: u8) -> bool {
    (STATUS_PORT.read() & bit) != 0
}

/// Represents a PS2 controller command without a return value
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ControllerCommand {
    WriteConfig = 0x60,
    DisablePort2 = 0xA7,
    EnablePort2 = 0xA8,
    DisablePort1 = 0xAD,
    EnablePort1 = 0xAE,
    WriteInputPort2 = 0xD4,
}

/// Represents a PS2 controller command with a return value
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum ControllerReturnCommand {
    ReadConfig = 0x20,
    TestController = 0xAA,
    TestPort1 = 0xAB,
    TestPort2 = 0xA9,
    IdentifyDevice = 0xF2,
}

/// Represents a PS2 device command opcode
#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum DeviceCommand {
    EnableScanning = 0xF4,
    DisableScanning = 0xF5,
    SetDefaults = 0xF6,
    SetScancode = 0xF0,
    Reset = 0xFF,
}
