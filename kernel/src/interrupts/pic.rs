// Thanks to http://www.randomhacks.net/2015/11/16/bare-metal-rust-configure-your-pic-interrupts/

use io::SynchronizedPort;
use spin::Mutex;

pub static CHAINED_PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new((0x20, 0x28)));

/// Used to pause execution temporarily
pub static IO_WAIT_PORT: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x80) };

const COMMAND_INIT: u8 = 0x10 | 0x1;
const COMMAND_END_OF_INTERRUPT: u8 = 0x20;

// TODO: Handle spurious interrupts

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

    pub fn initialize(&mut self) {
        // Tell the PIC to initialize
        self.write_command(COMMAND_INIT);

        // Set the PICs offset
        self.write_data(self.offset);
    }

    pub fn write_command(&mut self, command: u8) {
        self.command_port.write(command);
        IO_WAIT_PORT.write(0x0);
    }

    pub fn write_data(&mut self, value: u8) {
        self.data_port.write(value);
        IO_WAIT_PORT.write(0x0);
    }

    pub fn read_data(&self) -> u8 {
        let value = self.data_port.read();
        IO_WAIT_PORT.write(0x0);
        value
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
                             SynchronizedPort::new(0x21),
                    ),
                    Pic::new(
                        offsets.1,
                        SynchronizedPort::new(0xA0),
                        SynchronizedPort::new(0xA1),
                    ),
                ],
            }
        }
    }

    /// Initializes the PICs and remaps them to avoid IRQ overlaps
    pub fn init_and_remap(&mut self) {
        // Cache chip masks
        let mask_1 = self.inner[0].read_data();
        let mask_2 = self.inner[1].read_data();

        IO_WAIT_PORT.write(0x0);

        // Tell both of the PICs to initialise
        for pic in self.inner.iter_mut() {
            pic.initialize();
        }

        self.inner[0].write_data(4);
        self.inner[1].write_data(2);

        // Set PICs to 8086/88 (MCS-80/85) mode
        for pic in self.inner.iter_mut() {
            pic.write_data(0x1);
        }

        // Restore chip masks
        self.inner[0].write_data(mask_1);
        self.inner[1].write_data(mask_2);
    }

    /// Masks all interrupts on both PICs. This will cause all fired interrupts to be ignored
    pub fn disable(&mut self) {
        for pic in self.inner.iter_mut() {
            pic.write_data(0xFF);
        }
    }

    pub fn handle_interrupt<F: FnOnce()>(&mut self, irq: u8, handle: F) -> Result<(), ()> {
        handle();
        self.end_of_interrupt(irq)
    }

    pub fn end_of_interrupt(&mut self, irq: u8) -> Result<(), ()> {
        if let Some((target_pic, local_irq)) = self.line_target(irq) {
            target_pic.write_command(COMMAND_END_OF_INTERRUPT);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn enable_line(&mut self, irq: u8) -> Result<(), ()> {
        if let Some((target_pic, local_irq)) = self.line_target(irq) {
            let mask = target_pic.read_data();
            target_pic.write_data(mask & !(1 << local_irq));
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn disable_line(&mut self, irq: u8) -> Result<(), ()> {
        if let Some((target_pic, local_irq)) = self.line_target(irq) {
            let mask = target_pic.read_data();
            target_pic.write_data(mask | (1 << local_irq));
            Ok(())
        } else {
            Err(())
        }
    }

    fn line_target(&mut self, irq: u8) -> Option<(&mut Pic, u8)> {
        // IRQs 0-7 go to the master PIC, while 8-15 go to the slave
        if irq < 8 {
            Some((&mut self.inner[0], irq))
        } else if irq < 16 {
            Some((&mut self.inner[1], irq - 8))
        } else {
            None
        }
    }
}
