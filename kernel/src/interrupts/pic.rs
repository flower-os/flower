// Thanks to http://www.randomhacks.net/2015/11/16/bare-metal-rust-configure-your-pic-interrupts/

use io::SynchronizedPort;
use interrupts;
use spin::Mutex;

pub static CHAINED_PICS: Mutex<ChainedPics> = Mutex::new(ChainedPics::new());

/// Used to pause execution temporarily
pub static IO_WAIT_PORT: SynchronizedPort<u8> = unsafe { SynchronizedPort::new(0x80) };

const COMMAND_INIT: u8 = 0x11;
const COMMAND_END_OF_INTERRUPT: u8 = 0x20;

const COMMAND_READ_IRR: u8 = 0x0A;
const COMMAND_READ_ISR: u8 = 0x0B;

/// Represents an 8295/8295A PIC (superseded by APIC)
struct Pic {
    offset: u8,
    command_port: SynchronizedPort<u8>,
    data_port: SynchronizedPort<u8>,
}

impl Pic {
    const fn new(offset: u8, command_port: SynchronizedPort<u8>, data_port: SynchronizedPort<u8>) -> Self {
        Pic { offset, command_port, data_port }
    }

    fn initialize(&mut self) {
        // Tell the PIC to initialize
        self.write_command(COMMAND_INIT);

        // Set the PICs offset
        self.write_data(self.offset);
    }

    fn end_of_interrupt(&mut self) {
        self.write_command(COMMAND_END_OF_INTERRUPT);
    }

    fn enable_line(&mut self, irq: u8) {
        let mask = self.read_data();
        self.write_data(mask & !(1 << irq));
    }

    fn disable_line(&mut self, irq: u8) {
        let mask = self.read_data();
        self.write_data(mask | (1 << irq));
    }

    /// Fetches the value of the interrupt request register
    fn irr(&self) -> u8 {
        self.write_command(COMMAND_READ_IRR);
        self.command_port.read()
    }

    /// Fetches the value of the in-service register
    fn isr(&self) -> u8 {
        self.write_command(COMMAND_READ_ISR);
        self.command_port.read()
    }

    fn is_spurious(&self, irq: u8) -> bool {
        (self.isr() & (1 << irq)) == 0
    }

    fn write_command(&self, command: u8) {
        self.command_port.write(command);
        IO_WAIT_PORT.write(0x0);
    }

    fn write_data(&mut self, value: u8) {
        self.data_port.write(value);
        IO_WAIT_PORT.write(0x0);
    }

    fn read_data(&self) -> u8 {
        let value = self.data_port.read();
        IO_WAIT_PORT.write(0x0);
        value
    }
}

/// Represents two [Pic]s chained together as they are in the hardware
pub struct ChainedPics {
    master: Pic,
    slave: Pic,
}

impl ChainedPics {
    pub const fn new() -> Self {
        unsafe {
            ChainedPics {
                master: Pic::new(
                    0x20,
                    SynchronizedPort::new(0x20),
                    SynchronizedPort::new(0x21),
                ),
                slave: Pic::new(
                    0x28,
                    SynchronizedPort::new(0xA0),
                    SynchronizedPort::new(0xA1),
                ),
            }
        }
    }

    /// Initializes the PICs and remaps them to avoid IRQ overlaps
    pub fn init_and_remap(&mut self) {
        // Cache chip masks
        let master_mask = self.master.read_data();
        let slave_mask = self.slave.read_data();

        IO_WAIT_PORT.write(0x0);

        // Tell both of the PICs to initialise
        self.master.initialize();
        self.slave.initialize();

        self.master.write_data(4);
        self.slave.write_data(2);

        // Set PICs to 8086/88 (MCS-80/85) mode
        self.master.write_data(0x1);
        self.slave.write_data(0x1);

        // Restore chip masks
        self.master.write_data(master_mask);
        self.slave.write_data(slave_mask);
    }

    /// Masks all interrupts on both PICs. This will cause all fired interrupts to be ignored
    pub fn disable(&mut self) {
        self.master.write_data(0xFF);
        self.slave.write_data(0xFF);
    }

    pub fn handle_interrupt(&mut self, irq: u8, handler: fn()) {
        match self.destination(irq) {
            IrqDestination::Master(_) => {
                if !self.master.is_spurious(irq) {
                    handler();
                    self.master.end_of_interrupt();
                }
            },
            IrqDestination::Slave(_) => {
                if !self.slave.is_spurious(irq) {
                    handler();
                    self.slave.end_of_interrupt();
                } else {
                    self.master.end_of_interrupt();
                }
            },
        }
    }

    pub fn enable_line(&mut self, irq: u8) {
        match self.destination(irq) {
            IrqDestination::Master(local_irq) => self.master.enable_line(local_irq),
            IrqDestination::Slave(local_irq) => self.slave.enable_line(local_irq),
        }
    }

    pub fn disable_line(&mut self, irq: u8) {
        match self.destination(irq) {
            IrqDestination::Master(local_irq) => self.master.disable_line(local_irq),
            IrqDestination::Slave(local_irq) => self.slave.disable_line(local_irq),
        }
    }

    fn destination(&mut self, irq: u8) -> IrqDestination {
        // IRQs 0-7 go to the master PIC, while 8-15 go to the slave
        if irq < 8 {
            IrqDestination::Master(irq & 7)
        } else if irq < 16 {
            IrqDestination::Slave(irq & 7)
        } else {
            panic!("irq {} out of bounds, expected range 0-15", irq);
        }
    }
}

enum IrqDestination {
    Master(u8),
    Slave(u8),
}
