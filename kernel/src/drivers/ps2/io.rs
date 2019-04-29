//! Internal, low-level IO with PS/2 ports

use crate::io::Port;
use crate::drivers::pit;
use crate::drivers::ps2;

/// The amount of time in milliseconds to wait for IO access before terminating
pub const BLOCK_TIME_MS: usize = 10;

/// Port used to send data to the controller, and for the controller to return responses
pub static DATA_PORT: Port<u8> = unsafe { Port::new(0x60) };
/// Port used to check controller status and to send commands to the controller
pub static CONTROLLER_PORT: Port<u8> = unsafe { Port::new(0x64) };

/// Port used to pause through IO
pub static IO_WAIT_PORT: Port<u8> = unsafe { Port::new(0x80) };

bitflags! {
    struct StatusFlags: u8 {
        /// If the output buffer from the controller is full (data can be read)
        const OUTPUT_FULL = 1 << 0;
        /// If the input buffer to the controller is full (data cannot be written)
        const INPUT_FULL = 1 << 1;
        /// If the current output from the controller is from the second port
        const PORT_2_FULL = 1 << 5;
    }
}

/// Writes to the given port, and blocks until available. Panics if the output status bit is never
/// unset within a time because that should never happen.
pub fn write(port: &Port<u8>, value: u8) {
    // Iterate until maximum time reached or write available
    let cancel_time = pit::time_ms() + BLOCK_TIME_MS;
    while pit::time_ms() < cancel_time {
        IO_WAIT_PORT.write(0x0);
        if can_write() {
            port.write(value);
            return;
        }
    }
    panic!("Writing to PS/2 controller took too long!");
}

/// Reads from the given port, or blocks until a response has been sent back. Returns `None` if
/// timeout is exceeded while awaiting a response.
pub fn read(port: &Port<u8>) -> Option<u8> {
    // Iterate until maximum time reached or response available
    let cancel_time = pit::time_ms() + BLOCK_TIME_MS;
    while pit::time_ms() < cancel_time {
        IO_WAIT_PORT.write(0x0);
        if can_read() {
            return Some(port.read());
        }
    }
    None
}

/// Flushes the controller's output buffer, discarding any bytes in the buffer
pub fn flush_output() {
    // Read until the output status bit is empty
    while can_read() {
        IO_WAIT_PORT.write(0x0);
        DATA_PORT.read();
    }
}

/// Reads from the status port and returns the flags
fn read_status() -> StatusFlags {
    StatusFlags::from_bits_truncate(CONTROLLER_PORT.read())
}

/// Returns true if the write status bit is 0
fn can_write() -> bool {
    !read_status().contains(StatusFlags::INPUT_FULL)
}

/// Returns true if the read status bit is 1
pub fn can_read() -> bool {
    read_status().contains(StatusFlags::OUTPUT_FULL)
}

pub trait CommandIo {
    fn send(command: u8) -> ps2::Result<()>;
    fn send_data(command: u8, data: u8) -> ps2::Result<()>;

    fn read() -> ps2::Result<u8>;
}
