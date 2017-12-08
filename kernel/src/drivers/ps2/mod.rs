pub mod io;

use drivers::ps2::io::*;
use spin::Mutex;

pub const RESEND: u8 = 0xFE;
pub const ACK: u8 = 0xFA;

lazy_static! {
    pub static ref PS2: Mutex<Ps2Controller> = Mutex::new(Ps2Controller::new());
}

bitflags! {
    pub struct Ps2ConfigFlags: u8 {
        const PORT_INTERRUPT_1 = 1 << 0;
        const PORT_INTERRUPT_2 = 1 << 1;
        const SYSTEM = 1 << 2;
        const KEYBOARD_LOCK = 1 << 4;
        const PORT_OUTPUT_FULL_2 = 1 << 5;
        const PORT_TRANSLATION_1 = 1 << 6;
    }
}

/// Represents the PS2 master controller
pub struct Ps2Controller {
    pub initialized: bool,
    pub devices: [Ps2Device; 2],
    config: Ps2ConfigFlags,
}

impl Ps2Controller {
    fn new() -> Self {
        Ps2Controller {
            initialized: false,
            devices: [
                Ps2Device::new(DevicePort::Keyboard),
                Ps2Device::new(DevicePort::Mouse),
            ],
            config: Ps2ConfigFlags::empty(),
        }
    }

    /// Initializes this PS2 controller
    pub fn initialize(&mut self) -> Result<(), Ps2Error> {
        println!("ps2c: initializing");

        for device in self.devices.iter_mut() {
            device.disable()?;
            device.state = DeviceState::Unavailable;
        }

        println!("ps2c: disabled devices");

        flush_output()?;

        self.initialize_config()?;

        if !self.test_controller()? {
            println!("ps2c: controller test failed");
        }

        // Test the first device
        self.devices[0].test()?;

        // Check if controller supports the second device
        if !self.config.contains(Ps2ConfigFlags::PORT_OUTPUT_FULL_2) {
            self.devices[1].enable()?;
            self.read_config()?;
            self.devices[1].disable()?;
        }

        // Test the second device
        if self.config.contains(Ps2ConfigFlags::PORT_OUTPUT_FULL_2) {
            self.devices[1].test()?;
        } else {
            println!("ps2c: second device unsupported");
        }

        let mut available_count: u8 = 0;
        for device in self.devices.iter_mut() {
            // Enable if device available
            if device.state == DeviceState::Available {
                device.enable()?;
                device.reset()?;
                available_count += 1;
            }
        }

        // Check if any devices are available
        if available_count > 0 {
            println!("ps2c: enabled devices");
        } else {
            println!("ps2c: detected no available devices");
        }

        flush_output()?;

        self.initialized = true;

        Ok(())
    }

    /// Initializes the config for this controller
    fn initialize_config(&mut self) -> Result<(), Ps2Error> {
        // Read the config from the controller
        self.read_config()?;

        // Set all required config flags
        self.config.set(Ps2ConfigFlags::PORT_INTERRUPT_1, false);
        self.config.set(Ps2ConfigFlags::PORT_INTERRUPT_2, false);
        self.config.set(Ps2ConfigFlags::PORT_TRANSLATION_1, false);

        // Write the updated config back to the controller
        self.write_config()?;

        println!("ps2c: initialized config");

        Ok(())
    }

    /// Tests this controller
    fn test_controller(&mut self) -> Result<bool, Ps2Error> {
        Ok(command_ret(ControllerReturnCommand::TestController)? == 0x55)
    }

    /// Writes the current config to the PS2 controller
    pub fn write_config(&mut self) -> Result<(), Ps2Error> {
        command_data(ControllerCommand::WriteConfig, self.config.bits())
    }

    /// Reads the config from the PS2 controller
    pub fn read_config(&mut self) -> Result<(), Ps2Error> {
        let read = command_ret(ControllerReturnCommand::ReadConfig)?;
        self.config = Ps2ConfigFlags::from_bits_truncate(read);

        Ok(())
    }
}

/// Represents a PS2 device
pub struct Ps2Device {
    state: DeviceState,
    port: DevicePort,
}

impl Ps2Device {
    const fn new(port: DevicePort) -> Self {
        Ps2Device {
            state: DeviceState::Unavailable,
            port,
        }
    }

    /// Tests this device to see if it is available
    pub fn test(&mut self) -> Result<bool, Ps2Error> {
        let available = command_ret(if self.port == DevicePort::Mouse {
            ControllerReturnCommand::TestPort2
        } else {
            ControllerReturnCommand::TestPort1
        })? == 0x0;

        if available {
            self.state = DeviceState::Available;
        } else {
            self.state = DeviceState::Unavailable;
        }

        Ok(available)
    }

    /// Enables this device
    pub fn enable(&mut self) -> Result<(), Ps2Error> {
        if self.state == DeviceState::Available {
            command(if self.port == DevicePort::Mouse {
                ControllerCommand::EnablePort2
            } else {
                ControllerCommand::EnablePort1
            })?;
            self.state = DeviceState::Enabled;
        }
        Ok(())
    }

    /// Disables this device
    pub fn disable(&mut self) -> Result<(), Ps2Error> {
        command(if self.port == DevicePort::Mouse {
            ControllerCommand::DisablePort2
        } else {
            ControllerCommand::DisablePort1
        })?;
        self.state = DeviceState::Available;
        Ok(())
    }

    /// Resets this device
    pub fn reset(&mut self) -> Result<(), Ps2Error> {
        self.command(DeviceCommand::Reset)?;

        Ok(())
    }

    /// Sends a command for this PS2 device and returns result
    #[allow(dead_code)] // Used by drivers depending on PS/2
    pub fn command(&mut self, cmd: DeviceCommand) -> Result<u8, Ps2Error> {
        // If second PS2 port, send context switch command
        if self.port == DevicePort::Mouse {
            command(ControllerCommand::WriteInputPort2)?;
        }
        for _ in 0..4 {
            write_data(cmd as u8)?;
            let result = read_data()?;
            if result != RESEND {
                return Ok(result)
            }
        }
        Err(Ps2Error::NoData)
    }

    /// Sends a command for this PS2 device with data and returns a result
    #[allow(dead_code)] // Used by drivers depending on PS/2
    pub fn command_data(&mut self, cmd: DeviceCommand, data: u8) -> Result<u8, Ps2Error> {
        self.command(cmd).and_then(|result| {
            if result == ACK {
                write_data(data as u8)?;
                read_data()
            } else {
                Ok(result)
            }
        })
    }
}

/// Represents the state of a device.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DeviceState {
    /// The device does not exist or hasn't been detected yet
    Unavailable,
    /// The device has been detected but is not enabled
    Available,
    /// The device has been enabled
    Enabled,
}

/// Represents the port of a device
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DevicePort {
    /// The device is in the keyboard port
    Keyboard,
    /// The device is in the mouse port
    Mouse,
}
