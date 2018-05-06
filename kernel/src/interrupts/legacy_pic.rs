// Thanks to http://www.randomhacks.net/2015/11/16/bare-metal-rust-configure-your-pic-interrupts/

use io::SynchronizedPort;
use spin::Mutex;

pub static CHAINED_PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new((0x20, 0x28)));

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

    unsafe fn end_of_interrupt(&self) {
        self.command_port.write(Commands::EndOfInterrupt as u8);
    }

    pub fn initialise(&self) {
        // Port used to pause execution temporarily
        let wait_port: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x80) };

        // Tell the PIC to initialise
        self.command_port.write(Commands::Init as u8);
        wait_port.write(0);

        // Set the PICs offset
        self.data_port.write(self.offset);
        wait_port.write(0);
    }

    pub fn set_mode(&self, mode: u8) {
        // Port used to pause execution temporarily
        let wait_port: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x80) };

        self.data_port.write(mode);
        wait_port.write(0);
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
        // Port used to pause execution temporarily
        let wait_port: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x80) };

        wait_port.write(0);

        // Tell both of the PICs to initialise
        for pic in self.inner.iter() {
            pic.initialise();
        }

        for (pic, data) in self.inner.iter().zip([4u8, 2u8].iter()) {
            pic.data_port.write(*data);
            wait_port.write(0);
        }

        // Set PICs to 8086/88 (MCS-80/85) mode
        for pic in self.inner.iter() {
            pic.set_mode(0x1);
        }

        // Mask pics
        self.inner[0].data_port.write(0xFF);
        self.inner[1].data_port.write(0xFF);
    }
}
