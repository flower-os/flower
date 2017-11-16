use x86_64::instructions::port;

/// Represents a port to be accessed through inb and outb
pub struct IOPort {
    port: u16,
}

impl IOPort {
    pub const unsafe fn new(port: u16) -> Self {
        IOPort {
            port: port,
        }
    }

    /// Writes a byte to this port
    pub fn write(&self, value: u8) {
        unsafe { port::outb(self.port, value) }
    }

    /// Reads a byte from this port
    pub fn read(&self) -> u8 {
        unsafe { port::inb(self.port) }
    }
}
