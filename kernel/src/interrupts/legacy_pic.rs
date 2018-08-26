// Thanks to http://www.randomhacks.net/2015/11/16/bare-metal-rust-configure-your-pic-interrupts/

use io::SynchronizedPort;
use spin::Mutex;

pub static CHAINED_PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new((0x20, 0x28)));

/// Used to pause execution temporarily
pub static IO_WAIT_PORT: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x80) };

#[repr(u8)]
enum Commands {
    Init = 0x10,
    EndOfInterrupt = 0x20,
}

/// Represents an 8295/8295A PIC (superseded by APIC)
pub struct Pic {
    pub offset: u8,
    pub command_port: SynchronizedPort<u8>,
    pub data_port: SynchronizedPort<u8>,
}

impl Pic {
    const fn new(offset: u8, command_port: SynchronizedPort<u8>, data_port: SynchronizedPort<u8>) -> Self {
        Pic { offset, command_port, data_port }
    }

    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }

    fn end_of_interrupt(&self) {
        self.command_port.write(Commands::EndOfInterrupt as u8);
    }

    pub fn initialise(&self) {
        // Tell the PIC to initialise
        self.command_port.write(Commands::Init as u8);
        IO_WAIT_PORT.write(0x0);

        // Set the PICs offset
        self.write(self.offset);
    }

    #[inline]
    pub fn write(&self, value: u8) {
        self.data_port.write(value);
        IO_WAIT_PORT.write(0x0);
    }
}

/// Represents two [Pic]s chained together as they are in the hardware
pub struct ChainedPics {
    inner: [Pic; 2],
}

impl ChainedPics {
    pub const fn new(offsets: (u8, u8)) -> Self {
        unsafe {
            ChainedPics {
                inner: [
                    Pic::new(offsets.0,
                             SynchronizedPort::new(0x20),
                             SynchronizedPort::new(0x21)
                    ),
                    Pic::new(
                        offsets.1,
                        SynchronizedPort::new(0xA0),
                        SynchronizedPort::new(0xA1)
                    ),
                ],
            }
        }
    }

    pub fn remap_and_disable(&self) {
        IO_WAIT_PORT.write(0x0);

        // Tell both of the PICs to initialise
        for pic in self.inner.iter() {
            pic.initialise();
        }

        self.inner[0].write(4);
        self.inner[1].write(2);

        // Set PICs to 8086/88 (MCS-80/85) mode
        for pic in self.inner.iter() {
            pic.write(0x1);
        }

        // Mask pics
        for pic in self.inner.iter() {
            pic.data_port.write(0xFF);
        }
    }
}
