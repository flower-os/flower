//! Thanks to https://en.wikibooks.org/wiki/Serial_Programming/8250_UART_Programming and OSDev wiki

use core::fmt::{self, Write};
use spin::Mutex;
use crate::io::Port;

pub const PORT_1_ADDR: u16 = 0x3f8;
pub const PORT_2_ADDR: u16 = 0x2f8;
pub const MAX_BAUD: u32 = 115200;

pub static PORT_1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(PORT_1_ADDR) });
pub static PORT_2: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(PORT_2_ADDR) });

pub struct SerialPort {
    initialized: bool,
    data: Port<u8>,
    interrupt_enable: Port<u8>,
    fifo_control: Port<u8>,
    line_control: Port<u8>,
    modem_control: Port<u8>,
    line_status: Port<u8>,
    modem_status: Port<u8>,
    scratch: Port<u8>,
}

impl SerialPort {
    pub const unsafe fn new(port_base: u16) -> SerialPort {
        SerialPort {
            initialized: false,
            data: Port::new(port_base),
            interrupt_enable: Port::new(port_base + 1),
            fifo_control: Port::new(port_base + 2),
            line_control: Port::new(port_base + 3),
            modem_control: Port::new(port_base + 4),
            line_status: Port::new(port_base + 5),
            modem_status: Port::new(port_base + 6),
            scratch: Port::new(port_base + 7),
        }
    }

    /// Initializes the serial port
    pub fn init(&mut self, baud: u32, enable_irqs: bool) -> Result<(), InvalidBaudrate> {
        let divisor = MAX_BAUD / baud;
        if MAX_BAUD / divisor != baud {
            return Err(InvalidBaudrate(baud));
        }

        // Disable interrupts
        self.interrupt_enable.write(0);

        // Enable DLAB - data port & interrupt enable will temporarily become DLAB lsb & msb
        self.line_control.write(1 << 7);

        // Write divisor
        self.data.write((divisor & 0xFF) as u8);
        self.interrupt_enable.write((divisor >> 8) as u8);

        // 8 bits, no parity byte, one stop bit
        self.line_control.write(0b111);

        //             Flags: Enable     & Reset tx/rx & 64byte buf & trigger level 56bytes
        let fifo_flags = (0b1 << 0) | (0b11 << 1) | (0b1 << 5) | (0b11 << 6);
        self.fifo_control.write(fifo_flags);

        //  Request To Send & Data Terminal Ready
        let mut modem_ctrl_flags = 0b1 << 0;

        if enable_irqs {
            modem_ctrl_flags |= 0b1 << 3; // Aux output 2 (enable IRQs, practically)
        }

        self.modem_control.write(modem_ctrl_flags);

        self.initialized = true;
        Ok(())
    }

    /// Returns the line status. Panics if not initialized.
    pub fn status(&mut self) -> LineStatus {
        if self.initialized {
            LineStatus::from_bits_truncate(self.modem_status.read())
        } else {
            panic!("Serial port not initialized");
        }
    }

    /// Attempts to read one byte of data, returning none if no byte was waiting.
    pub fn try_read(&mut self) -> Option<u8> {
        if self.status().contains(LineStatus::DATA_READY) {
            Some(self.data.read())
        } else {
            None
        }
    }

    /// Attempts to write one byte of data, returning whether it could.
    pub fn try_write(&mut self, data: u8) -> bool {
        if self.status().contains(LineStatus::TRANSMITTER_HOLDING_REGISTER_EMPTY) {
            self.data.write(data);
            true
        } else {
            false
        }
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            while !self.try_write(byte) {}
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct InvalidBaudrate(u32);

bitflags! {
    pub struct LineStatus: u8 {
        const DATA_READY = 1 << 0;
        const OVERRUN_ERROR = 1 << 1;
        const PARITY_ERROR = 1 << 2;
        const FRAMING_ERROR = 1 << 3;
        const BREAK_INDICATOR = 1 << 4;
        const TRANSMITTER_HOLDING_REGISTER_EMPTY = 1 << 5;
        const TRANSMITTER_EMPTY = 1 << 6;
        const IMPENDING_ERROR = 1 << 7;
    }
}
