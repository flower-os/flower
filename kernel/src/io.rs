/// Reads u8 from given port
pub unsafe fn inb(port: u16) -> u8 {
    let inb: u8;
    asm!("inb %dx, %al" : "={ax}"(inb) : "{dx}"(port) :: "volatile");
    return inb;
}

/// Writes a u8 to the given port
pub unsafe fn outb(port: u16, value: u8) {
    asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value));
}

/// Represents a port to be accessed through inb and outb
pub struct IOPort {
    port: u16,
}

impl IOPort {
    pub const fn new(port: u16) -> Self {
        IOPort {
            port: port,
        }
    }

    /// Writes a byte to this port
    pub fn write(&self, value: u8) {
        unsafe { outb(self.port, value) }
    }

    /// Reads a byte from this port
    pub fn read(&self) -> u8 {
        unsafe { inb(self.port) }
    }
}
