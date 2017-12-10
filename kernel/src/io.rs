use core::marker::PhantomData;

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

//Read word from port.
pub unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    asm!("inw %dx, %ax" : "={ax}"(result) : "{dx}"(port) :: "volatile");
    result
}

///Write a word to the port.
pub unsafe fn outw(value: u16, port: u16) {
    asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(value) :: "volatile");
}

///Read a dword from the port.
pub unsafe fn inl(port: u16) -> u32 {
    let result: u32;
    asm!("inl %dx, %eax" : "={eax}"(result) : "{dx}"(port) :: "volatile");
    result
}

///Write a dword to the port.
pub unsafe fn outl(value: u32, port: u16) {
    asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(value) :: "volatile");
}

pub trait InOut {
    unsafe fn port_in(port: u16) -> Self;
    unsafe fn port_out(port: u16) -> Self;
}

//TODO add type implementors.

/// Represents a port to be accessed through in/out instructions. The values read and written are
/// `InOut` in size.
#[derive(Debug)]
pub struct IOPort<T: InOut>{
    //Allows for very high port numbers.
    port: u16,
    //Zero size type placeholder.
    phantom: PhantomData<T>,
}

impl<T: InOut> IOPort<T> {
    pub const unsafe fn new(port: u16) -> IOPort<T> {
        IOPort {
            port: port,
            phantom: PhantomData,
        }
    }

    /// Writes a value to this port
    pub fn write(&mut self, value: T) {
        unsafe {
            T::port_out(self.port, value);
        }
    }

    /// Reads a value from this port
    pub fn read(&mut self) -> T {
        unsafe { T::port_in(self.port) }
    }
}
