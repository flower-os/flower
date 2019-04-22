use core::marker::PhantomData;
use spin::{Mutex, MutexGuard};

pub mod x86_io {
    /// Read a single byte from the port.
    pub unsafe fn inb(port: u16) -> u8 {
        let result: u8;
        asm!("inb %dx, %al" : "={al}"(result) : "{dx}"(port) :: "volatile");
        result
    }

    /// Write a single byte to the port.
    pub unsafe fn outb(value: u8, port: u16) {
        asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }

    /// Read a word from the port.
    pub unsafe fn inw(port: u16) -> u16 {
        let result: u16;
        asm!("inw %dx, %ax" : "={ax}"(result) : "{dx}"(port) :: "volatile");
        result
    }

    /// Write a word to the port.
    pub unsafe fn outw(value: u16, port: u16) {
        asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(value) :: "volatile");
    }

    /// Read a dword from `port`.
    pub unsafe fn inl(port: u16) -> u32 {
        let result: u32;
        asm!("inl %dx, %eax" : "={eax}"(result) : "{dx}"(port) :: "volatile");
        result
    }

    /// Write a dword to the `port`.
    pub unsafe fn outl(value: u32, port: u16) {
        asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(value) :: "volatile");
    }
}

use self::x86_io::{inb, inl, inw, outb, outl, outw};

/// Nice little type that allows us to specify the size of the value read without using inb
/// directly.
pub trait InOut {
    unsafe fn port_in(port: u16) -> Self;
    unsafe fn port_out(port: u16, value: Self);
}

impl InOut for u8 {
    unsafe fn port_in(port: u16) -> u8 {
        inb(port)
    }
    unsafe fn port_out(port: u16, value: u8) {
        outb(value, port);
    }
}

impl InOut for u16 {
    unsafe fn port_in(port: u16) -> u16 {
        inw(port)
    }
    unsafe fn port_out(port: u16, value: u16) {
        outw(value, port);
    }
}

impl InOut for u32 {
    unsafe fn port_in(port: u16) -> u32 {
        inl(port)
    }
    unsafe fn port_out(port: u16, value: u32) {
        outl(value, port);
    }
}

/// An `InOut`sized port. This could be any of the type implementors for `InOut`.
#[derive(Debug)]
pub struct Port<T: InOut> {
    /// Port address.
    port: u16,

    /// Zero-byte placeholder.  This is only here so that we can have a
    /// type parameter `T` without a compiler error.
    phantom: PhantomData<T>,
}

impl<T: InOut> Port<T> {
    /// Create a port which can handle values of `T` size.
    pub const unsafe fn new(port: u16) -> Port<T> {
        Port {
            port,
            phantom: PhantomData,
        }
    }

    /// Read a value from `self.port`.
    pub fn read(&mut self) -> T {
        unsafe { T::port_in(self.port) }
    }

    /// Write a value to `self.port`.
    pub fn write(&mut self, value: T) {
        unsafe { T::port_out(self.port, value); }
    }
}

/// An `InOut` sized port that is synchronized using a spinlock. See [Port]
pub struct SynchronizedPort<T: InOut> {
    inner: Mutex<Port<T>>,
}

impl<'a, T: InOut> SynchronizedPort<T> {
    ///Create a port which can handle values of `T` size.
    pub const unsafe fn new(port: u16) -> SynchronizedPort<T> {
        SynchronizedPort {
            inner: Mutex::new(Port::new(port))
        }
    }

    /// Read a value from `self.port`. Synchronized over context of this read.
    pub fn read(&self) -> T {
        self.inner.lock().read()
    }

    /// Write a value to `self.port`. Synchronized over context of this write.
    #[allow(dead_code)] // Part of API
    pub fn write(&self, value: T) {
        self.inner.lock().write(value)
    }

    /// Operates a closure on the synchronized port. Synchronized over the whole context of the
    /// closure.
    pub fn with_lock<R, F: FnOnce(MutexGuard<'a, Port<T>>) -> R>(&'a self, f: F) -> R {
        f(self.inner.lock())
    }

    /// Locks the port and returns a mutex guard over the port
    pub fn lock(&'a self) -> MutexGuard<'a, Port<T>> {
        self.inner.lock()
    }
}